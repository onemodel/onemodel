/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004, 2008-2019 inclusive, 2022-2023 inclusive, and 2025-2025 inclusive, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
//%% use std::env;
// use crate::controllers::controller::Controller;
use crate::util::Util;
use crate::color::Color;
use console::{Key, Term};
/* (Some possible alternatives if ever needed, to rustyline?
    www.rust-lang.org/what/cli ... thing to read up on?
    reedline
    others at crates.io as things change over time (search for "readline" maybe and at libs.rs &c.)
*/
use rustyline::error::ReadlineError;
// use rustyline::{Editor, Result as RustyLineResult};
use rustyline::{DefaultEditor, Result as RustyLineResult};
//use std::error::Error;
// use std::io::{Error, ErrorKind};

pub struct TextUI {
    //%%read up something in rust book, see examples: better way to handle it? how instantiate w/o all its vars being pub?
    pub testing: bool,
}

impl TextUI {
    //i.e., for the "n-" menu number prefix on each option shown in "ask_which":
    const MENU_CHARS: &'static str =
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
    const CHOOSER_MENU_PREFIX_LENGTH: u16 = 2;

    pub fn how_quit(&self) -> &'static str {
        if Util::is_windows() {
            "Close the window"
        } else {
            "Ctrl+C"
        }
    }

    //idea: is there a more idiomatic way?:
    fn we_are_testing(&mut self, testing_in: bool) {
        self.testing = testing_in;
    }

    fn are_we_testing(&self) -> bool {
        self.testing
    }

    pub fn display_text1(&self, text: &str) {
        self.display_text2(text, true);
    }
    pub fn display_text2(&self, text: &str, wait_for_keystroke: bool) {
        self.display_text3(text, wait_for_keystroke, None);
    }
    pub fn display_text3(&self, text: &str, wait_for_keystroke: bool, pre_prompt: Option<String>) {
        TextUI::display_visual_separator();
        println!("{}", text);

        if wait_for_keystroke && (!self.testing) {
            print!("{}", pre_prompt.unwrap_or(String::from("")));
            println!("Press any key to continue...");
            TextUI::wait_for_user_input_key();
        }
    }

       /// The # of items (or lines, actually) to try to display on the screen at one time.
    fn terminal_height() -> u16 {
        //*******NOTE: changes here should probably also made simililarly in terminal_width()
        //idea: put this behind a Once thing so it only gives the error once!?
        let result: std::io::Result<(u16,u16)> = termion::terminal_size();
        match result {
            Ok((_cols, rows)) => rows.into(),
            Err(e) => {
                eprintln!("Unable to determine terminal dimensions; using 80x25 as default. Error was: {}",
                    e.to_string());
                25
            },
        }
      }
    fn terminal_width() -> u16 {
        //*******NOTE: changes here should probably also made simililarly in terminal_width()
        //idea: put this behind a Once thing so it only gives the error once!? and saves
        //recalculating each time.
        let result: std::io::Result<(u16,u16)> = termion::terminal_size();
        match result {
            Ok((cols, _rows)) => cols.into(),
            Err(_e) => {
                //eprintln!("Unable to determine terminal dimensions; using 80x25 as default. Error was: {}",
                //    e.to_string());
                80
            },
        }
        //old/reference in case this doesnt work on Windows properly? was that just a jline2 bug
        //that is eliminated w/ the move to rust and termion?:
        //if !Util::is_windows() {
        //  mTerminal.get_width()
        //} else {
        //  // This is a not-ideal workaround to a bug when running on Windows (at least when OM was written in Scala and
        //  // using jline2), where OM thinks it has more terminal width than it does: in a 95-character-wide
        //  // command window, OM was displaying all entity name characters, including all the padding spaces 
        //  // for multiple om-display columns (ie, 2 if the entity names are short & a 2-columns list will fit), 
        //  // up to 160 wide.  This caused an entity with a 5-character name to take up two lines,
        //  // so the list looked like there was a blank line between each entry in the list: ugly.
        //  // A better solution would be to take time to see what is the real bug, and if that can be fixed, 
        //  // or if necessary just disable the spaces (name padding for columns) and having multiple 
        //  // columns, on windows (I hardly ever use multiple columns of entries on Linux or openbsd, 
        //  // and I'm not sure it's working right anyway).
        //  // This # seems likely to fit in a customized command window on an 800x600 display:
        //  93
        //}
      }

    fn wait_for_user_input_key() {
        //was: TextUI::get_user_input_char1(None)
        //was, temporarily, a way to do w/ std that requires pressing Enter:
        // use std::io;
        // let mut input=String::new();
        // match io::stdin().read_line(&mut input) {
        //     Ok(n) => {
        //         // println!("{} bytes input", n);
        //         println!("input: {}", input);
        //     }
        //     Err(e) => {
        //         eprintln!("error: {}", e);
        //     }
        // }

        // The Result (even Err) from this doesn't currently matter(?), just that the user pressed
        // any key.
        let _r = Term::stdout().read_key();
    }

    /** Returns the key pressed and whether it was an alt key combo (ESC key combination).
     */
    pub fn get_user_input_char() -> Result<(char, bool), std::io::Error> {
        //allowed_chars_in_CURRENTLY_IGNORED: Option<Vec<char>>,
        //%%fix this to use the ignored parm just above, or eliminate it or the method? what did it
        //do in scala? change to just "idea"?
        let term = Term::stdout();
        let key_read = term.read_key()?;
        let mut alt_combo = false;

        let key: char = match &key_read {
            Key::UnknownEscSeq(keystrokes) => {
                alt_combo = true;
                if keystrokes.len() != 1 {
                    dbg!(keystrokes.len(), keystrokes);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        //idea: add more info, the key to err msg. use Display trait methods of the char, and +?
                        "Unexpected key(s) pressed?  Expected a length of 1.",
                    ));
                }
                keystrokes[0]
            }
            Key::Char(keystroke) => *keystroke,
            Key::Escape => {
                let o: Option<char> = std::char::from_u32(27);
                if o.is_some() {
                    o.unwrap_or_default()
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        //idea: add more info, the key to err msg. use Display trait methods of the char, and +?
                        "Unexpected state: from_u32(27) returned None.",
                    ));
                }
            }
            Key::Enter => {
                let o: Option<char> = std::char::from_u32(32);
                if o.is_some() {
                    o.unwrap_or_default()
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        //idea: add more info, the key to err msg. use Display trait methods of the char, and +?
                        "Unexpected state: from_u32(32) returned None.",
                    ));
                }
            }
            _ => {
                println!("unexpected key pressed:  {:?}", key_read);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    //idea: add more info, the key to err msg. use Display trait methods of the char, and +?
                    "unexpected key pressed?  This function expects ASCII.",
                ));
            }
        };

        // show user what they just did, in a visually simple way (I find this often useful)
        match &key_read {
            Key::UnknownEscSeq(_) => {
                println!("Alt+{}", key);
            }
            Key::Char(_) => println!("{}", key),
            _ => println!("{:?}", key_read),
        };
        // some chars, like Ctrl+D, are shown just above as a space. So...?:
        // println!("{:?}", key_read);

        // idea: Is there a test for this method? use, now?
        Ok((key, alt_combo))
    }

    /* %%
      /** Allows customizing the output stream, for tests.
        */
        fn println() {
        out.println()
      }

        fn println(s: String) {
        out.println(s)
      }
      let mut out: PrintStream = System.out;
      fn setOutput(out: PrintStream) {
        this.out = out
      }
        fn initializeReader() -> ConsoleReader {
                         let is: InputStream = if inIn.isEmpty { System.in } else { inIn.get };
                         jlineReader.setBellEnabled(false)
                         //handy sometimes:
                         //jlineReader.setDebug(new PrintWriter(System.err))
                         // allow ESC to abort an editing session (in combination w/ jline2 version / modifications):
                         let startingKeyMap: String = jlineReader.getKeyMap;
                         jlineReader.setKeyMap(jline.console.KeyMap.EMACS_META)
                         jlineReader.getKeys.bind(jline.console.KeyMap.ESCAPE.toString, jline.console.Operation.QUIT)
                         jlineReader.setKeyMap(KeyMap.VI_MOVE)
                         jlineReader.getKeys.bind(jline.console.KeyMap.ESCAPE.toString, jline.console.Operation.QUIT)
                         jlineReader.setKeyMap(startingKeyMap)
                         jlineReader
    */

    fn display_visual_separator() {
        for _ in 1..6 {
            println!();
        }
        println!("==============================================")
    }

    pub fn ask_for_string1(&self, leading_text: Vec<&str>) -> Option<String> {
        //TextUI::ask_for_string5(self, leading_text, None, "", false, true)
        self.ask_for_string5(leading_text, None, "", false, true)
    }
    pub fn ask_for_string3(
        &self,
        leading_text: Vec<&str>,
        criteria: Option<fn(s: &str/*, ui: &TextUI*/) -> bool>,
        default_value: &str,
    ) -> Option<String> {
        //TextUI::ask_for_string5(self, leading_text, criteria, default_value, false, true)
        self.ask_for_string5(leading_text, criteria, default_value, false, true)
    }
    pub fn ask_for_string4(
        &self,
        leading_text: Vec<&str>,
        criteria: Option<fn(s: &str/*, ui: &TextUI*/) -> bool>,
        default_value: &str,
        is_password: bool,
    ) -> Option<String> {
        TextUI::ask_for_string5(
            self,
            leading_text,
            criteria,
            default_value,
            is_password,
            true,
        )
    }
    /// Returns the string entered (None if the user just wants out of this question or whatever, 
    /// unless escKeySkipsCriteriaCheck is false).
    /// The parameter "criteria"'s Option is a function which takes a String (which will be the user 
    /// input), which it checks for validity.
    /// If the entry didn't meet the criteria, it repeats the question until it does or user gets out w/ ESC.
    /// A simple way to let the user know why it didn't meet the criteria is to put them in the leading text.
    /// The same-named functions with fewer parameters default to, after the first: None, None, false, true, respectively.
    //%%@tailrec //see below note on 'recursive' for why removed 4 now.
    pub fn ask_for_string5(
        &self,
        leading_text_in: Vec<&str>,
        // idea: is there a way to make this Option take a closure or function, instead of
        // just a function, as suggested by "The Rust Programming Language" ("the Book"),
        // where it mentions "FnMut"?
        criteria_in: Option<fn(s: &str/*(not needed right?), ui: &TextUI*/) -> bool>,
        default_value_in: &str,
        //%%use this parm below, not just as another parameter. For pwd entry, sch crates.ui for "password entry" and/or use dialoguer and/or can rustyline do it/modify it/ask somewhere anyway? (see also other %%cmts below re this like "dialoguer" etc)
        is_password_in: bool,
        esc_key_skips_criteria_check_in: bool,
    ) -> Option<String> {
        let mut count = 0;
        let last_line_of_prompt: String = {
            let mut last_line_of_prompt = String::new();
            let num_prompt_lines = leading_text_in.len();
            for prompt in leading_text_in {
                count = count + 1;
                if count < num_prompt_lines {
                    // all but the last one
                    println!("{}", prompt);
                } else {
                    last_line_of_prompt = format!("prompt{}", ": ");
                }
            }
            last_line_of_prompt
        };
        // idea: make this any better by using features of the ~ readline library? Or...?  At least make it
        // easier to see when out of room?
        //val promptToShowStringSizeLimit = "(Max name length is  " + Controller.maxNameLength
        let end_prompt = "(... which ends here: |)";
        if last_line_of_prompt.chars().count() > 1
            && last_line_of_prompt.chars().count() + end_prompt.chars().count() - 1
                <= Util::max_name_length() as usize
        {
            let mut spaces: String = String::new();
            // (the + 1 in next line is for the closing parenthesis in the prompt, which comes after the visual end position marker in end_prompt.
            let pad_length: u32 = Util::max_name_length()
                - last_line_of_prompt.chars().count() as u32
                - end_prompt.chars().count() as u32
                + 1;
            for _ in 0..pad_length {
                spaces.push(' ')
            }
            println!("{}{}{}", last_line_of_prompt, spaces, end_prompt)
        } else {
            println!("{last_line_of_prompt}");
        }

        let user_input: Option<String> = loop {
            // `()` can be used when no completer is required?
            //let r = Editor::<()>::new();
            let r = DefaultEditor::new();
            match r {
                Err(e) => {
                    eprintln!(
                        "Unable to create line editor in ask_for_string5():  {}",
                        e.to_string()
                    );
                    break None;
                }
                Ok(mut editor) => {
                    //%%how to make ESC exit the prompt and return None as some things expect!??
                    // then try that w/ username/password forced w/ x parm.
                    //%%why did blank pwd not give any err nor exit? try gdb or how best2debug? (was when util.get_default_user_login alw returned a bad def pwd)
                    //%%see if the editor history has password in it? is there any edi hist or have2specify?see docs
                    //%%add password mask -- use dialoguer crate? termion (says it can; alr using?)? 
                    //or ask/ck issue tracker for rustyline?
                    // let line = jlineReader.readLine(null, if is_password_in { '*' } else { null } );
                    //%%make ^C work to get out of prompt! ?  see where trapped ("Error: ..."), just below.
                    let line = editor.readline_with_initial("", (default_value_in, ""));
                    match line {
                        Ok(l) => { 
                            if l.len() == 0 && esc_key_skips_criteria_check_in {
                                break None;
                            } else {
                                match criteria_in {
                                    None => break Some(l),
                                    Some(check_criteria_function) => {
                                        //%%see if ESC does get user out (or blank)? I.e., test use 
                                        //of esc_key_skips_criteria_check_in ? is it called w/ that 
                                        //parm = true, ever? fr where--test that? See cmts below at
                                        //end of the fn also.
                                        //How to make ESC exit the prompt and return None as some things expect!??
                                        // then try that w/ username/password forced w/ x parm.
                                        //%%make ^C work to get out of prompt?  see where trapped ("Error: ..."), just below.
                                        if check_criteria_function(l.as_str()) {
                                            break Some(l);
                                        } else {
                                            self.display_text1(
                                                "Didn't pass the criteria; please re-enter.",
                                            );
                                            continue;
                                        }
                                    }
                                }
                            }
                        },
                        Err(ReadlineError::Interrupted) => {
                            println!("CTRL-C");
                            // user wants out.
                            // %%but shouldn't controller be doing that? pass none back instead or
                            // what? or just reqd here to make ^C work?
                            std::process::exit(1);
                        }
                        Err(err) => {
                            eprintln!("Error: {:?}", err);
                            break None;
                        }
                    }
                }
            };
        };

        //%%just debugging probably
        let x = user_input.clone().unwrap_or("None".to_string());
        let y = format!("{}", x);
        println!("user input: {}", y);
        //return Some(y);

        return user_input;
    }

        /// after accounting for leading text and the initial choices.
        fn lines_left(num_of_leading_text_lines_in: usize, num_choices_above_columns_in: usize) -> usize {
            // 5 as described in one caller
            let lines_used_before_more_choices: usize = num_of_leading_text_lines_in + num_choices_above_columns_in + 5;
            Self::terminal_height() as usize - lines_used_before_more_choices
        }

      /// Calculate the maximum number of columnar choices that can be displayed
      /// after accounting for other lines.
      ///
      /// I.e., the # of attributes ("moreChoices" elsewhere) that will likely fit in the space available on the
      /// screen AFTER the preceding leading_text lines + menu size + 5 (5 because: 1 line added by 
      /// ask_which(...) (for the 0/ESC menu option), 1 line for the visual separator,
      /// and 1 line for the cursor at the bottom to not push things off the top, and 2 more 
      /// because entity/group names and the line that shows them at the
      /// top of a menu are long & wrap, so they were still pushing things off the top of the visual 
      /// space (could have made it 3 more for small windows, but that
      /// might make the list of data too short in some cases, and 2 is probably usually enough 
      /// if windows aren't too narrow)).
      ///
      /// Based on # of available columns and a possible max column width.
      /// SEE ALSO the method lines_left(), which actually has/uses the number.
    fn max_columnar_choices_to_display_after(
        &self,
        num_of_leading_text_lines_in: usize,
        num_choices_above_columns_in: usize,
        field_width_in: usize,
    ) -> usize {
        let max_more_choices_by_space_available = 
            TextUI::lines_left(num_of_leading_text_lines_in, num_choices_above_columns_in) 
            * TextUI::columns_possible(field_width_in + usize::from(TextUI::CHOOSER_MENU_PREFIX_LENGTH));
        // The next 2 lines are in coordination with an 'assert!' statement in ask_which_choice_or_its_alternate, 
        // so we don't fail it:
        let max_more_choices_by_menu_chars_available = TextUI::MENU_CHARS.len();
        std::cmp::min(max_more_choices_by_space_available, max_more_choices_by_menu_chars_available)
    }

    fn columns_possible(column_width: usize) -> usize {
        //idea: instead of assert!, would it be better to make a check, output a message, and
        //return 0?
        assert!(column_width > 0);
        // allow at least 1 column, even with a smaller terminal width
        std::cmp::max(usize::from(TextUI::terminal_width()) / column_width, 1)
      }

    /// The parm "choices" are shown in a single-column list; the "moreChoices" are shown in columns as 
    /// space allows. The return value is either None (if user just wants out), or Some(the # of the result 
    /// chosen) (1-based, where the index is against the *combined* choices and moreChoices).  
    /// Ex., if the choices parameter has 3 elements, and moreChoices has 5, the
    /// return value can range from 1-8 (1-based, not 0-based!).
    /// If calling methods are kept small, it should be easy for them to visually determine which 'choice's 
    /// go with the return value;
    /// see current callers for examples of how to easily determine which 'moreChoice's go with the return value.
    /// 
    /// highlightIndexIn 0-based (like almost everything; exceptions are noted.).
    /// secondaryHighlightIndexIn 0-based.
    /// defaultChoiceIn 1-based.
    /// return value is 1-based (see description).
    pub fn ask_which(
        &self,
        leading_text_in: Option<Vec<&str>>,
        choices_in: Vec<&str>,
        more_choices_in: Vec<&str>, 
        include_esc_choice_in: bool, /* = true*/
        trailing_text_in: Option<&str>, /* = None*/
        highlight_index_in: Option<usize>, /* = None*/
        secondary_highlight_index_in: Option<usize>, /* = None*/
        default_choice_in: Option<usize>, /* = None*/
    ) -> Option<usize> {
        let result = self.ask_which_choice_or_its_alternate(
            leading_text_in,
            choices_in,
            more_choices_in,
            include_esc_choice_in,
            trailing_text_in,
            highlight_index_in,
            secondary_highlight_index_in,
            default_choice_in,
        );

        if result.is_none() {
            None
        } else {
            Some(result.unwrap().0)
        }
    }

    fn show_choices(line_count: &mut i32, already_full_in: &mut bool, choices_in: &Vec<&str>,
        more_choices_in: &Vec<&str>,
        last_menu_chars_index: &mut i32, all_allowed_answers: &mut String, possible_menu_chars: &str,
        default_choice_in: &Option<usize>, include_esc_choice_in: &bool) 
    {
        // see containing method description: these choices are 1-based when considered from the human/UI perspective:
        let mut index: usize = 1;

        for choice in choices_in {
            if !Self::ran_out_of_vertical_space(line_count, already_full_in, &choices_in, more_choices_in) {
                let menu_char = Self::next_menu_char(last_menu_chars_index, 
                    all_allowed_answers, &possible_menu_chars);
                let default_text = if default_choice_in.is_some() && index == default_choice_in.unwrap() {
                    "/Enter"
                } else {
                    ""
                };
                println!("{}{}-{}", menu_char, default_text, choice);
            }
            index += 1;
        }
        if *include_esc_choice_in && !Self::ran_out_of_vertical_space(line_count, already_full_in, &choices_in, 
            &more_choices_in) {
            println!("0/ESC - back/previous menu");
        }
    }

    fn show_more_choices(line_count: &mut i32, already_full_in: &mut bool, choices_in: &Vec<&str>,
        more_choices_in: &Vec<&str>, leading_text_in: &Option<Vec<&str>>, highlight_index_in: &Option<usize>,
        secondary_highlight_index_in: &Option<usize>, max_choice_length: &u32, all_allowed_answers: &mut String, 
        possible_menu_chars: &String, last_menu_chars_index: &mut i32) 
    {
        //claude added this condition? makes no sense.:
        //if let Some(more_choices) = &more_choices_in {
        if more_choices_in.len() > 0 {
            // This collection size might be much larger than needed (given possible multiple columns of display) 
            // but that's better than having more complex calculations
            let mut more_lines: Vec<String> = vec![String::new(); more_choices_in.len()];
            
            let num_of_leading_text_lines: usize = {
                leading_text_in.as_ref().map_or(0, |v| v.len()) 
            };
            let lines_left_here = Self::lines_left(
                num_of_leading_text_lines,
                choices_in.len()
            );
            
            let mut line_counter_local: usize = 0;
            // Now build the lines out of columns before displaying them
            let mut index: usize = 0;
            
            for choice in more_choices_in {
                if line_counter_local >= lines_left_here {
                    // 1st is 0-based, 2nd is 1-based
                    line_counter_local = 0; // Wraps to next column
                }
                // Not explicitly putting extra space between columns, because space can be in short supply,
                // and probably some of the choices will be shorter than the max length, to provide enough 
                // visual alignment/separation anyway. But make them equal length:
                let line_marker = if highlight_index_in.is_some() && highlight_index_in.unwrap() == index {
                    Color::blue(&"*".to_string())
                } else if secondary_highlight_index_in.is_some() && secondary_highlight_index_in.unwrap() == index {
                    Color::green(&"+".to_string())
                } else {
                    " ".to_string()
                };
                let pad_length = max_choice_length 
                    - u32::try_from(choice.len()).unwrap() 
                    - u32::try_from(TextUI::CHOOSER_MENU_PREFIX_LENGTH).unwrap()
                    - 1;
                let menu_char = Self::next_menu_char(last_menu_chars_index, 
                    all_allowed_answers, &possible_menu_chars);
                let line = format!("{}{}-{}", line_marker, menu_char, choice);
                more_lines[line_counter_local] += &line;
                for _ in 0..pad_length {
                    more_lines[line_counter_local] += " ";
                }
                index += 1;
                line_counter_local += 1;
            }
            
            let mut lines_too_long = false;
            for line in more_lines {
                if !line.trim().is_empty() && !Self::ran_out_of_vertical_space(line_count, already_full_in, &choices_in,
                    &more_choices_in) 
                {
                    // Idea for bugfix: adjust the effective_line_length for non-displaying chars
                    // that make up the color of the lineMarker above! Would need better
                    // multi-column tests?
                    let effective_line_length = line.trim().len();
                    if effective_line_length > usize::from(Self::terminal_width()) {
                        lines_too_long = true;
                    }
                    
                    let term_width = Self::terminal_width();
                    let line_to_display = if line.len() > usize::from(term_width) {
                        // (Appending Color.reset to the string in case it got cut with the substring 
                        // (0..term_width) cmd, which would allow the color to bleed to subsequent lines.)
                        //format!("{}{}", &line[0..term_width], Color::reset())
                        let part = Util::substring_from_start(&line, term_width.into());
                        format!("{}{}", part, Color::reset())
                    } else {
                        //format!("{}{}", line, Color::reset())
                        line
                    };
                    
                    println!("{}", line_to_display);
                }
            }
            
            if lines_too_long {
                // This could mean that either the terminal width is less than
                // Util::max_name_length() characters, or that there is a bug in the display logic:
                println!("(Some lines were longer than the terminal width and have been truncated.)");
            }
        }
    }
    
    fn next_menu_char(last_index: &mut i32, all_answers: &mut String, possible_chars: &str) -> String {
        let next = *last_index + 1;
        *last_index = next;
        if next as usize >= possible_chars.len() {
            return "(ran out)".to_string();
        }
        let next_char: char = possible_chars.chars().nth(next as usize).unwrap();
        all_answers.push(next_char);
        next_char.to_string()
    }

    fn ran_out_of_vertical_space(line_count: &mut i32, already_full_in: &mut bool,
        choices_in: &Vec<&str>, more_choices_in: &Vec<&str>) -> bool {
        *line_count += 1;
        if *already_full_in {
            return *already_full_in;
        } else if *line_count > Self::terminal_height().into() {
            // (+ 1 above to leave room for the error message line, below)
            let unshown_count: i32 = 
                (choices_in.len() + more_choices_in.len()) as i32 - *line_count - 1;
            eprintln!("==============================");
            eprintln!("FYI: Unable to show remaining {} items in the available screen space(!?). Consider code \
                    change to pass the right number of them, relaunching w/ larger terminal, or grouping \
                    things?  (ref: {}/{}/{}/{})",
                    unshown_count, already_full_in, line_count, Self::terminal_height(),
                    Self::terminal_width());

            //not failing after all (setting this to false causes ExpectIt tests to fail when run in IDE)
            eprintln!("Not going to fail over this, but it might be fixed, especially if you can reproduce \
                    it consistently.");
            eprintln!("==============================");
            //*already_full_in = true 
            return *already_full_in;
        } 
        false
    }

    /// Like ask_which but if user makes the alternate action on a choice (eg, double-click, click+differentButton,
    /// right-click, presses "alt+letter"), then it tells you so in the 2nd (boolean) part of the return value.
    pub fn ask_which_choice_or_its_alternate(
        &self,
        leading_text_in: Option<Vec<&str>>,
        choices_in: Vec<&str>,
        more_choices_in: Vec<&str>,
        include_esc_choice_in: bool, /* = true*/
        trailing_text_in: Option<&str>, /* = None*/
        highlight_index_in: Option<usize>, /* = None*/
        secondary_highlight_index_in: Option<usize>, /* = None*/
        default_choice_in: Option<usize>, /* = None*/
    ) -> Option<(usize, bool)> {
        // This attempts to always use as menu option keystroke choices: numbers for "choices" 
        // (such as major operations available on the current entity) and letters for "moreChoices" 
        // (such as attributes of the current entity to select for further work).  But if there 
        // are too many "choices", it will use letters for those as well.
        // I.e., 2nd part of menu ("moreChoices") always starts with a letter, not a #, but the 
        // 1st part can use numbers+letters as necessary. This is for the user experience: it 
        // seems will be easier to remember how to get around one's own model if attributes always 
        // start with 'a' and go from there.
        
        assert!(!choices_in.is_empty());
        let max_choice_length = Util::max_name_length();
        
        let mut first_menu_chars = String::new();
        for number in 1..=9 {
            if number <= choices_in.len() {
                //first_menu_chars.push(char::from_digit(number as u32, 10).unwrap());
                first_menu_chars.push_str(&number.to_string());
            }
        }
        let possible_menu_chars = format!("{}{}", first_menu_chars, TextUI::MENU_CHARS);
        // Make sure caller didn't send more than the # of things we can handle
        if (choices_in.len() + more_choices_in.len()) > possible_menu_chars.len() {
            panic!("Programming error: there are more choices provided ({}) than the menu can handle ({})",
                   choices_in.len() + more_choices_in.len(), possible_menu_chars.len());
        }

        let mut already_full = false;
        let mut line_counter: i32 = 0;
        let mut all_allowed_answers = String::new();
        let mut last_menu_chars_index: i32 = -1;

        TextUI::display_visual_separator();
        if let Some(leading_text) = &leading_text_in {
            if !leading_text.is_empty() {
                for prompt in leading_text {
                    line_counter += 1;
                    println!("{prompt}");
                }
            }
        }
        Self::show_choices(&mut line_counter, &mut already_full, &choices_in, &more_choices_in, &mut last_menu_chars_index,
            &mut all_allowed_answers, &possible_menu_chars, &default_choice_in, &include_esc_choice_in);
        Self::show_more_choices(&mut line_counter, &mut already_full, &choices_in, &more_choices_in, &leading_text_in,
            &highlight_index_in, &secondary_highlight_index_in, &max_choice_length, &mut all_allowed_answers, 
            &possible_menu_chars, &mut last_menu_chars_index);
        if let Some(trailing_text) = trailing_text_in {
            if !trailing_text.is_empty() {
                println!("{trailing_text}");
            }
        }

        let result: Result<(char, bool), std::io::Error> = Self::get_user_input_char();
        match result {
            Err(e) => {
                TextUI::display_visual_separator();
                eprintln!("Error getting input character: {}", e.to_string());
                self.ask_which_choice_or_its_alternate(
                    leading_text_in,
                    choices_in,
                    more_choices_in,
                    include_esc_choice_in,
                    trailing_text_in,
                    highlight_index_in,
                    secondary_highlight_index_in,
                    default_choice_in,
                )
            },
            Ok((answer, user_chose_alternate)) => { 
                if u32::from(answer) != 27 && answer != '0' && u32::from(answer) != 13 
                    && !all_allowed_answers.contains(answer) 
                {
                    println!("unknown choice: {}", answer as char);
                    self.ask_which_choice_or_its_alternate(
                        leading_text_in,
                        choices_in,
                        more_choices_in,
                        include_esc_choice_in,
                        trailing_text_in,
                        highlight_index_in,
                        secondary_highlight_index_in,
                        default_choice_in,
                    )
                } else if u32::from(answer) == 13 && (default_choice_in.is_some() || highlight_index_in.is_some()) {
                    // User hit Enter i.e. '\r', so take the one that was passed in as default, or highlighted
                    if default_choice_in.is_some() {
                        Some((default_choice_in.unwrap(), user_chose_alternate))
                    } else {
                        Some((choices_in.len() + highlight_index_in.unwrap() + 1, user_chose_alternate))
                    }
                } else if include_esc_choice_in && (answer == '0' || u32::from(answer) == 27) {
                    None
                } else {
                    // Result from this function is 1-based, but 'answer' is 0-based
                    let index = possible_menu_chars.find(answer as char).unwrap() + 1;
                    Some((index, user_chose_alternate))
                }
            }
        }
    }

    fn is_valid_yes_no_answer(s: &str) -> bool {
        s.to_lowercase() == "y"
            || s.to_lowercase() == "yes"
            || s.to_lowercase() == "n"
            || s.to_lowercase() == "no"
    }

    fn is_valid_yes_no_or_blank_answer(s: &str) -> bool {
        Self::is_valid_yes_no_answer(s) || s.trim().is_empty()
    }

    /// true means yes, None means user wants out.
    pub fn ask_yes_no_question(
        &self,
        prompt_in: String,
        default_value_in: &str,   /*= Some("n")*/
        allow_blank_answer: bool, /*= false*/
    ) -> Option<bool> {
        let answer = self.ask_for_string5(
            vec![format!("{} (y/n)", prompt_in).as_str()],
            if allow_blank_answer {
                Some(Self::is_valid_yes_no_or_blank_answer)
            } else {
                Some(Self::is_valid_yes_no_answer)
            },
            default_value_in,
            false,
            allow_blank_answer,
        );
        match answer {
            None if allow_blank_answer => None,
            Some(ans) if allow_blank_answer && ans.trim().is_empty() => None,
            Some(ans) => {
                if ans.to_lowercase().starts_with("y") {
                    Some(true)
                } else {
                    Some(false)
                }
            }
            _ => Some(false),
        }
    }

    /*%%
      /** This is in the UI code because probably a GUI would do it very differently.
        */
      fn getExportDestination(originalPathIn: String, originalMd5HashIn: String) -> Option[File] {
        fn newLocation(originalNameIn: String) -> Option[File] {
          let oldNameInTmpDir: File = new File(System.getProperty("java.io.tmpdir"), originalNameIn);
          if oldNameInTmpDir.getParentFile.canWrite && !oldNameInTmpDir.exists()) Some(oldNameInTmpDir)
          else {
            let (base_name, extension) = Util::get_usable_filename(originalPathIn);
            // for rust, see crate:  temp-file and its doc/refs to others, and
            // https://docs.rs/temp-file/0.1.7/temp_file/struct.TempFile.html#method.with_prefix
            // ...and similar methods chained like in ex at top of page, to get right result.
            // and use the leak() method so it isn't deleted on drop.
            // (Or: could consider just writing to a file with a random portion in its name, but that
            // ignores all the improvements/lessons embodied in the temp-file or similar crate.)
            Some(File.createTempFile(base_name + "-", extension))
          }
        }
        let originalFile = new File(originalPathIn);
        let originalContainingDirectory = originalFile.getParentFile;
        let originalName = FilenameUtils.getBaseName(originalPathIn);
        let baseConfirmationMessage = "Put the file in the original location: \"" + originalPathIn + "\"";
        let restOfConfirmationMessage = " (No means put it in a new/temporary location instead.)";
        if originalContainingDirectory != null && originalContainingDirectory.exists && (!originalFile.exists)) {
          let ans = ask_yes_no_question(baseConfirmationMessage + "?" + restOfConfirmationMessage);
          if ans.isEmpty) None
          else {
            if ans.get) Some(originalFile)
            else newLocation(originalName)
          }
        } else {
          let yesExportTheFile: Option<bool> = {;
            if originalFile.exists) {
              if FileAttribute::md5_hash(originalFile) != originalMd5HashIn) Some(true)
              else {
                ask_yes_no_question("The file currently at " + originalPathIn + " is identical to the one stored.  Export anyway?  (Answering " +
                                 "'y' will still allow choosing whether to overwrite it or write to a new location instead.)")
              }
            } else Some(true)
          }
          if yesExportTheFile.isEmpty || !yesExportTheFile.get) None
          else {
            if originalContainingDirectory != null && !originalContainingDirectory.exists) {
              newLocation(originalName)
            } else {
              require(originalFile.exists, "Logic got here unexpectedly, to a point where the original file is expected to exist but does not: " + originalPathIn)
              let ans = ask_yes_no_question(baseConfirmationMessage + "\" (overwriting the current copy)?" + restOfConfirmationMessage);
              if ans.isEmpty) None
              else if ans.get) Some(originalFile)
              else newLocation(originalName)
            }
          }
        }
      }
    */
}
