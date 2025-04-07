/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004, 2008-2019 inclusive, and 2022-2023 inclusive, Luke A. Call.
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
use console::{Key, Term};
/* (Some possible alternatives if ever needed, to rustyline?
    www.rust-lang.org/what/cli ... thing to read up on?
    reedline
    others at crates.io as things change over time (search for "readline" maybe and at libs.rs &c.)
*/
// use rustyline::error::ReadlineError;
// use rustyline::{Editor, Result as RustyLineResult};
// use std::error::Error;
// use std::io::{Error, ErrorKind};

pub struct TextUI {
    //%%read up something in rust book, see examples: better way to handle it? how instantiate w/o all its vars being pub?
    pub testing: bool,
}

impl TextUI {
    //i.e., for the "n-" menu number prefix on each option shown in "ask_which":
    const MENU_CHARS: &'static str =
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
    const CHOOSER_MENU_PREFIX_LENGTH: i32 = 2;

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
            //%%should this say to press Enter, instead? at startup, at least? or alw?
            println!("Press any key to continue...");
            TextUI::wait_for_user_input_key();
        }
    }

    /* %%
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
      let mut out: PrintStream = System.out;
      fn setOutput(out: PrintStream) {
        this.out = out
      }

      /**
       * The # of items to try to display on the screen at one time.
       */
        fn terminalHeight -> Int {
        mTerminal.getHeight
      }

        fn terminalWidth -> Int {
        if !Util::isWindows {
          mTerminal.getWidth
        } else {
          // This is a not-ideal workaround to a bug when running on Windows, where OM thinks it has more terminal width than it does: in a 95-character-wide
          // command window, OM was displaying all entity name characters, including all the padding spaces for multiple om-display columns (ie, 2
          // if the entity names are short & a 2-columns list will fit), up to 160 wide.  This caused an entity with a 5-character name to take up two lines,
          // so the list looked like there was a blank line between each entry in the list: ugly.
          // A better solution would be to take time to see what is the real bug, and if that can be fixed, or if necessary just disable the spaces (name
          // padding for columns) and having multiple columns, on windows (I hardly ever use it on Linux or openbsd, and I'm not sure it's working right anyway).
          // This # seems likely to fit in a customized command window on an 800x600 display:
          93
        }
      }
    */

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
        //%% allowed_cars_in_CURRENTLY_IGNORED: Option<Vec<char>>,
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
    */

    fn display_visual_separator() {
        for _ in 1..6 {
            println!();
        }
        println!("==============================================")
    }

    pub fn ask_for_string1(&self, leading_text: Vec<&str>) -> Option<String> {
        TextUI::ask_for_string5(self, leading_text, None, "", false, true)
    }
    pub fn ask_for_string3(
        &self,
        leading_text: Vec<&str>,
        criteria: Option<fn(s: &str, ui: &TextUI) -> bool>,
        default_value: &str,
    ) -> Option<String> {
        TextUI::ask_for_string5(self, leading_text, criteria, default_value, false, true)
    }
    pub fn ask_for_string4(
        &self,
        leading_text: Vec<&str>,
        criteria: Option<fn(s: &str, ui: &TextUI) -> bool>,
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
    /// Returns the string entered (None if the user just wants out of this question or whatever, unless escKeySkipsCriteriaCheck is false).
    /// The parameter "criteria"'s Option is a function which takes a String (which will be the user input), which it checks for validity.
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
        criteria_in: Option<fn(s: &str, ui: &TextUI) -> bool>,
        default_value_in: &str,
        //%%use rest of parms
        //%%for pwd entry, sch crates.ui for "password entry" and/or use dialoguer and/or can rustyline do it/modify it/ask somewhere anyway?
        _is_password_in: bool,
        _esc_key_skips_criteria_check_in: bool,
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

        use rustyline::error::ReadlineError;
        // use rustyline::{Editor, Result};
        use rustyline::Editor;
        // `()` can be used when no completer is required
        // let initial_text = match default_value_in {
        //     None => "",
        //     Some(s) => s,
        // };
        let user_input: Option<String> = loop {
            let r = Editor::<()>::new();
            match r {
                Err(e) => {
                    eprintln!(
                        "Unable to create line editor in ask_for_string5():  {}",
                        e.to_string()
                    );
                    break None;
                }
                Ok(mut editor) => {
                    // let line = editor.readline_with_initial("", (initial_text, ""));
                    let line = editor.readline_with_initial("", (default_value_in, ""));
                    match line {
                        Ok(l) => match criteria_in {
                            None => break Some(l),
                            Some(check_criteria_function) => {
                                if check_criteria_function(l.as_str(), self) {
                                    break Some(l);
                                } else {
                                    self.display_text1(
                                        "Didn't pass the criteria; please re-enter.",
                                    );
                                    continue;
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
                            println!("Error: {:?}", err);
                            break None;
                        }
                    }
                }
            };
        };

        //%%see if ESC does get user out (or blank)?
        //%%test use of esc_key_skips_criteria_check_in ? is it called w/ that parm = true, ever? fr where--test that?
        // if line != null {
        //   fn checkCriteria(line: String) -> Option<String> {
        //     if criteria_in.isEmpty || criteria_in.get(line) {
        //       Some(line)
        //     } else {
        //       display_text("Didn't pass the criteria; please re-enter.")
        //       // this gets "recursive call not in tail position", until new version of jvm that allows scala2do it?
        //       ask_for_string(leading_text_in, criteria_in, default_value_in, is_password_in, esc_key_skips_criteria_check_in)
        //     }
        //   }
        //   if line.isEmpty && esc_key_skips_criteria_check_in {
        //     None
        //   } else {
        //     checkCriteria(line)
        //   }
        // }

        let x = user_input.unwrap_or("None".to_string());
        let y = format!("{}", x);
        println!("user input: {}", y); //%%
                                       // return user_input;
        return Some(y);

        //%%how to make ESC exit the prompt and return None as some things expect!??
        // then try that w/ username/password forced w/ x parm.
        //%%why did blank pwd not give any err nor exit? try gdb or how best2debug? (was when util.get_default_user_login alw returned a bad def pwd)
        //%%see if the editor history has password in it? is there any edi hist or have2specify?see docs
        //%%add password mask -- use dialoguer crate? or ask/ck issue tracker for rustyline?
        // let line = jlineReader.readLine(null, if is_password_in { '*' } else { null } );
        //%%make ^C work to get out of prompt! ?  see where trapped ("Error: ..."), just above.
    }

    /* 
        fn linesLeft(numOfLeadingTextLinesIn: Int, numChoicesAboveColumnsIn: Int) -> Int {
            let linesUsedBeforeMoreChoices = numOfLeadingTextLinesIn + numChoicesAboveColumnsIn + 5 // 5 as described in one caller;
            terminalHeight - linesUsedBeforeMoreChoices
        }

          /** The # of attributes ("moreChoices" elsewhere) that will likely fit in the space available on the
            * screen AFTER the preceding leading_text lines + menu size + 5: 1 line added by ask_which(...) (for the 0/ESC menu option), 1 line for the visual separator,
            * and 1 line for the cursor at the bottom to not push things off the top, and 2 more because entity/group names and the line that shows them at the
            * top of a menu are long & wrap, so they were still pushing things off the top of the visual space (could have made it 3 more for small windows, but that
            * might make the list of data too short in some cases, and 2 is probably usually enough if windows aren't too narrow).
            * based on # of available columns and a possible max column width.
            * SEE ALSO the method linesLeft, which actually has/uses the number.
            */
          fn maxColumnarChoicesToDisplayAfter(numOfLeadingTextLinesIn: Int, numChoicesAboveColumnsIn: Int, fieldWidthIn: Int) -> Int {
            let maxMoreChoicesBySpaceAvailable = linesLeft(numOfLeadingTextLinesIn, numChoicesAboveColumnsIn) * columnsPossible(fieldWidthIn + CHOOSER_MENU_PREFIX_LENGTH);
            // the next 2 lines are in coordination with a 'require' statement in ask_which, so we don't fail it:
            let maxMoreChoicesByMenuCharsAvailable = TextUI.menuCharsList.length;
            math.min(maxMoreChoicesBySpaceAvailable, maxMoreChoicesByMenuCharsAvailable)
          }

            fn columnsPossible(columnWidthIn: Int) -> Int {
            require(columnWidthIn > 0)
            // allow at least 1 column, even with a smaller terminal width
            math.max(terminalWidth / columnWidthIn, 1)
          }
    %%%%*/
/*
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
          final fn ask_which(leading_text_in: Option[Vec<String>],
                             choices_in: Vec<String>,
                             moreChoicesIn: Vec<String> = Array(),
                             includeEscChoiceIn: bool = true,
                             trailingTextIn: Option<String> = None,
                             highlightIndexIn: Option[Int] = None,
                             secondaryHighlightIndexIn: Option[Int] = None,
                             defaultChoiceIn: Option[Int] = None) -> Option[Int] {
            let result = ask_whichChoiceOrItsAlternate(leading_text_in, choices_in, moreChoicesIn, includeEscChoiceIn, trailingTextIn,;
                                                      highlightIndexIn, secondaryHighlightIndexIn, defaultChoiceIn)
            if result.isEmpty None
            else Some(result.get._1)
          }

          /** Like ask_which but if user makes the alternate action on a choice (eg, double-click, click+differentButton, right-click, presses "alt+letter"),
            * then it tells you so in the 2nd (boolean) part of the return value.
            * */
          @tailrec
          final fn ask_whichChoiceOrItsAlternate(leading_text_in: Option[Vec<String>],
                             choices_in: Vec<String>,
                             moreChoicesIn: Vec<String> = Array(),
                             includeEscChoiceIn: bool = true,
                             trailingTextIn: Option<String> = None,
                             highlightIndexIn: Option[Int] = None,
                             //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) within this method, below!
                             secondaryHighlightIndexIn: Option[Int] = None,
                             defaultChoiceIn: Option[Int] = None) -> Option[(Int, bool)] {
            // This attempts to always use as menu option keystroke choices: numbers for "choices" (such as major operations available on the
            // current entity) and letters for "moreChoices" (such as attributes of the current entity to select for further work).  But if
            // there are too many "choices", it will use letters for those as well.
            // I.e., 2nd part of menu ("moreChoices") always starts with a letter, not a #, but the 1st part can use numbers+letters as necessary.
            // This is for the user experience: it seems will be easier to remember how to get around one's own model if attributes always start with
            // 'a' and go from there.
            require(choices_in.length > 0)

            let maxChoiceLength = Util::maxNameLength;

            let firstMenuChars: StringBuffer = {;
              //up to: "123456789"
              let chars = new StringBuffer;
              for (number: Int <- 1 to 9) if number <= choices_in.length {
                chars.append(number)
              }
              chars
            }
            let possibleMenuChars = firstMenuChars + TextUI.menuCharsList;
            // make sure caller didn't send more than the # of things we can handle
            require((choices_in.length + moreChoicesIn.length) <= possibleMenuChars.length, "Programming error: there are more choices provided (" +
                                                                                       (choices_in.length + moreChoicesIn.length) + ") than the menu can handle (" +
                                                                                       possibleMenuChars.length + ")")

            let alreadyFull = false;
            let mut lineCounter: i32 = 0;
            let allAllowedAnswers = new StringBuffer;

            let mut lastMenuCharsIndex: i32 = -1;
            fn nextMenuChar() -> String {
              let next = lastMenuCharsIndex + 1;
              lastMenuCharsIndex = next
              if next > possibleMenuChars.length {
                return "(ran out)"
              }
              allAllowedAnswers.append(possibleMenuChars.charAt(next))
              new String("" + possibleMenuChars.charAt(next))
            }

            fn ranOutOfVerticalSpace() -> bool {
              lineCounter = lineCounter + 1
              if alreadyFull {
                alreadyFull
              } else if !alreadyFull) && lineCounter > terminalHeight {
                // (+ 1 above to leave room for the error message line, below)
                let unshownCount: i32 = choices_in.length + moreChoicesIn.length - lineCounter - 1;
                println("==============================")
                println("FYI: Unable to show remaining " + unshownCount + " items in the available screen space(!?). Consider code change to pass the " +
                "right number of them, relaunching w/ larger terminal, or grouping things?  (ref: " + alreadyFull + "/" + lineCounter + "/" +
                        terminalHeight + "/" + terminalWidth + "/" + mTerminal.getClass.getCanonicalName + ")")
                println("Not going to fail over this, but it might be fixed, especially if you can reproduce it consistently.")
                println("==============================")
                //alreadyFull = true //not failing after all (setting this to false causes ExpectIt tests to fail when run in IDE)
                alreadyFull
              } else false
            }

            fn showChoices() {
              // see containing method description: these choices are 1-based when considered from the human/UI perspective:
              let mut index: i32 = 1;

              for (choice <- choices_in) {
                if !ranOutOfVerticalSpace {
                  println(nextMenuChar() +
                          (if defaultChoiceIn.is_defined && index == defaultChoiceIn.get) "/Enter" else "" +
                          "-" + choice)
                }
                index += 1
              }
              if includeEscChoiceIn && !ranOutOfVerticalSpace {
                println("0/ESC - back/previous menu")
              }
            }

            fn showMoreChoices() {
              if moreChoicesIn.length == 0 {
                //noinspection ScalaUselessExpression (intentional style violation, for readability):
                Unit
              } else {
                // this collection size might be much larger than needed (given multiple columns of display) but that's better than having more complex calculations.
                let moreLines = new Array[StringBuffer](moreChoicesIn.length);
                for (i <- moreLines.indices) {
                  moreLines(i) = new StringBuffer()
                }
                let linesLeftHere = linesLeft(leading_text_in.size, choices_in.length);
                let mut lineCounter = -1;
                // now build the lines out of columns be4 displaying them.
                let mut index = -1;
                for (choice <- moreChoicesIn) {
                  index += 1
                  lineCounter = lineCounter + 1
                  if lineCounter >= linesLeftHere {
                    // 1st is 0-based, 2nd is 1-based
                    lineCounter = 0 //wraps to next column
                  }
                  // Not explicitly putting extra space between columns, because space can be in short supply, and probably some of the choices
                  // will be shorter than the max length, to provide enough visual alignment/separation anyway.  But make them equal length:
                  let lineMarker: String =;
                    if highlightIndexIn.getOrElse(None) == index { Color.blue("*") }
                    else if secondaryHighlightIndexIn.getOrElse(None) == index { Color.green("+") }
                    else { " " }
                  let padLength = maxChoiceLength - choice.length - CHOOSER_MENU_PREFIX_LENGTH - 1;
                  moreLines(lineCounter).append(lineMarker + nextMenuChar() + "-" + choice)
                  for (x <- 0 until padLength) {
                    moreLines(lineCounter).append(" ")
                  }
                }
                let mut linesTooLong = false;
                for (line <- moreLines) {
                  if line.toString.trim.length > 0 && !ranOutOfVerticalSpace {
                    // idea for bugfix: adjust the effectiveLineLength for non-displaying chars that make up the color of the lineMarker above!
                    let effectiveLineLength = line.toString.trim.length;
                    if effectiveLineLength > terminalWidth {
                      linesTooLong = true
                    }
                    // (Appending Color.reset to the string in case it got cut with the substring cmd, allowing the color to bleed to subsequent lines.)
                    println(line.toString.substring(0, math.min(line.length, terminalWidth)) + Color.reset)
                  }
                }
                if linesTooLong {
                  // This could mean that either the terminal width is less than " + Controller.maxNameLength + " characters, or
                  // that there is a bug in the display logic:
                  println("(Some lines were longer than the terminal width and have been truncated.)")
                }
              }
            }


            display_visual_separator()
            if leading_text_in.is_defined && leading_text_in.get.length > 0 {
              for (prompt <- leading_text_in.get) {
                lineCounter = lineCounter + 1
                println(prompt)
              }
            }
            showChoices()
            showMoreChoices()
            if trailingTextIn.is_defined && trailingTextIn.get.nonEmpty { println(trailingTextIn.get) }

            let (answer: Char, userChoseAlternate: bool) = get_user_input_char;
            if answer != 27 && answer != '0' && answer != 13 && (!allAllowedAnswers.toString.contains(answer.toChar)) {
              println("unknown choice: " + answer)
              ask_whichChoiceOrItsAlternate(leading_text_in, choices_in, moreChoicesIn, includeEscChoiceIn, trailingTextIn, highlightIndexIn, secondaryHighlightIndexIn,
                                           defaultChoiceIn)
            } else if answer == 13 && (defaultChoiceIn.is_defined() || highlightIndexIn.is_defined()) {
              // user hit Enter ie '\r', so take the one that was passed in as default, or highlighted
              if defaultChoiceIn.is_defined {
                Some(defaultChoiceIn.get, userChoseAlternate)
              } else {
                Some(choices_in.length + highlightIndexIn.get + 1, userChoseAlternate)
              }
            } else if includeEscChoiceIn && (answer == '0' || answer == 27) {
              None
            } else {
              Some(possibleMenuChars.indexOf(answer) + 1, userChoseAlternate) // result from this function is 1-based, but 'answer' is 0-based.
            }
          }
*/ //%%%%

/*
//use std::io::{stdin, stdout, Write};
//use std::string::ToString;

// Assuming these constants and functions exist elsewhere in your Rust code
// and are accessible here. You might need to adjust imports based on your
// actual project structure.
// const MAX_NAME_LENGTH: usize = 80; // Example value
// const CHOOSER_MENU_PREFIX_LENGTH: usize = 3; // Example value
// const MENU_CHARS_LIST: &str = "abcdefghijklmnopqrstuvwxyz"; // Example value
// const TERMINAL_HEIGHT: i32 = 25; // Example value
// const TERMINAL_WIDTH: i32 = 80; // Example value
// struct Color { /* ... */ }
// impl Color {
//     const BLUE: &'static str = "\x1b[34m";
//     const GREEN: &'static str = "\x1b[32m";
//     const RESET: &'static str = "\x1b[0m";
// }
// fn display_visual_separator() { /* ... */ }
// fn lines_left(leading_text_size: usize, choices_size: usize) -> i32 { /* ... */ }
// fn get_user_input_char() -> (char, bool) { /* ... */ }

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
    leading_text_in: Option<Vec<String>>,
    choices_in: Vec<String>,
    more_choices_in: Vec<String>,
    include_esc_choice_in: bool,
    trailing_text_in: Option<String>,
    highlight_index_in: Option<usize>,
    secondary_highlight_index_in: Option<usize>,
    default_choice_in: Option<usize>,
) -> Option<usize> {
    let result = ask_which_choice_or_its_alternate(
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

/// Like ask_which but if user makes the alternate action on a choice (eg, double-click,
/// click+differentButton, right-click, presses "alt+letter"), then it tells you so in the
/// 2nd (boolean) part of the return value.
pub fn ask_which_choice_or_its_alternate(
    leading_text_in: Option<Vec<String>>,
    choices_in: Vec<String>,
    more_choices_in: Vec<String>,
    include_esc_choice_in: bool,
    trailing_text_in: Option<String>,
    highlight_index_in: Option<usize>,
    secondary_highlight_index_in: Option<usize>,
    default_choice_in: Option<usize>,
) -> Option<(usize, bool)> {
    assert!(!choices_in.is_empty());

    // Assuming MAX_NAME_LENGTH is accessible.
    let max_choice_length = 80; // Util::maxNameLength;

    let mut first_menu_chars = String::new();
    for number in 1..=9 {
        if number <= choices_in.len() {
            first_menu_chars.push_str(&number.to_string());
        }
    }
    // Assuming MENU_CHARS_LIST is accessible.
    let possible_menu_chars = first_menu_chars + "abcdefghijklmnopqrstuvwxyz"; // TextUI::menuCharsList;
    assert!(
        (choices_in.len() + more_choices_in.len()) <= possible_menu_chars.len(),
        format!(
            "Programming error: there are more choices provided ({}) than the menu can handle ({})",
            choices_in.len() + more_choices_in.len(),
            possible_menu_chars.len()
        )
    );

    let mut already_full = false;
    let mut line_counter: i32 = 0;
    let mut all_allowed_answers = String::new();
    let mut last_menu_chars_index: i32 = -1;
}

    fn next_menu_char(
        last_menu_chars_index: &mut i32,
        possible_menu_chars: &str,
        all_allowed_answers: &mut String,
    ) -> String {
        let next = *last_menu_chars_index + 1;
        *last_menu_chars_index = next;
        if next >= possible_menu_chars.len() as i32 {
            return "(ran out)".to_string();
        }
        all_allowed_answers.push(possible_menu_chars.chars().nth(next as usize).unwrap());
        possible_menu_chars.chars().nth(next as usize).unwrap().to_string()
    }

    fn ran_out_of_vertical_space(
        line_counter: &mut i32,
        already_full: &mut bool,
        terminal_height: i32,
        choices_len: usize,
        more_choices_len: usize,
    ) -> bool {
        *line_counter += 1;
        if *already_full {
            return *already_full;
        } else if !*already_full && *line_counter > terminal_height {
            let unshown_count: i32 = choices_len as i32 + more_choices_len as i32 - *line_counter - 1;
            println!("==============================");
            println!(
                "FYI: Unable to show remaining {} items in the available screen space(!?). Consider code change to pass the right number of them, relaunching w/ larger terminal, or grouping things?  (ref: {}/{}/{}/{}/{})",
                unshown_count,
                *already_full,
                *line_counter,
                terminal_height,
                80, // terminalWidth, assuming it's 80
                "unknown_terminal_class" // mTerminal.getClass().getCanonicalName() - needs replacement
            );
            println!("Not going to fail over this, but it might be fixed, especially if you can reproduce it consistently.");
            println!("==============================");
            // *already_full = true; //not failing after all (setting this to false causes ExpectIt tests to fail when run in IDE)
            return *already_full;
        } else {
            false
        }
    }

    fn show_choices(
        choices_in: &Vec<String>,
        include_esc_choice_in: bool,
        default_choice_in: &Option<usize>,
        line_counter: &mut i32,
        already_full: &mut bool,
        terminal_height: i32,
        last_menu_chars_index: &mut i32,
        possible_menu_chars: &str,
        all_allowed_answers: &mut String,
    ) {
        let mut index: usize = 1;
        for choice in choices_in {
            if !ran_out_of_vertical_space(
                line_counter,
                already_full,
                terminal_height,
                choices_in.len(),
                0, // moreChoicesIn.length is 0 here
            ) {
                let menu_char = next_menu_char(
                    last_menu_chars_index,
                    possible_menu_chars,
                    all_allowed_answers,
                );
                let default_indicator = if default_choice_in.is_some() && index == default_choice_in.unwrap() {
                    "/Enter"
                } else {
                    ""
                };
                println!("{}-{}{}", menu_char, default_indicator, choice);
            }
            index += 1;
        }
        if include_esc_choice_in && !ran_out_of_vertical_space(
            line_counter,
            already_full,
            terminal_height,
            choices_in.len(),
            0, // moreChoicesIn.length is 0 here
        ) {
            println!("0/ESC - back/previous menu");
        }
    }

    fn show_more_choices(
        more_choices_in: &Vec<String>,
        highlight_index_in: &Option<usize>,
        secondary_highlight_index_in: &Option<usize>,
        line_counter: &mut i32,
        already_full: &mut bool,
        terminal_height: i32,
        choices_len: usize,
        leading_text_size: usize,
        last_menu_chars_index: &mut i32,
        possible_menu_chars: &str,
        all_allowed_answers: &mut String,
    ) {
        if more_choices_in.is_empty() {
            // noinspection ScalaUselessExpression (intentional style violation, for readability):
            // Unit in Scala is like doing nothing in Rust
        } else {
            let mut more_lines: Vec<String> = vec![String::new(); more_choices_in.len()];
            let lines_left_here = lines_left(leading_text_size, choices_len);
            let mut current_line_counter = -1;
            let mut index = -1;

            for choice in more_choices_in {
                index += 1;
                current_line_counter += 1;
                if current_line_counter >= lines_left_here {
                    current_line_counter = 0;
                }

                let line_marker: String = if highlight_index_in.is_some() && highlight_index_in.unwrap() == index {
                    "\x1b[34m*\x1b[0m".to_string() // Color::blue("*")
                } else if secondary_highlight_index_in.is_some() && secondary_highlight_index_in.unwrap() == index {
                    "\x1b[32m+\x1b[0m".to_string() // Color::green("+")
                } else {
                    " ".to_string()
                };

                let menu_char = next_menu_char(
                    last_menu_chars_index,
                    possible_menu_chars,
                    all_allowed_answers,
                );
                let pad_length = max_choice_length - choice.len() - 3 - 1; // CHOOSER_MENU_PREFIX_LENGTH = 3 assumed
                let mut padded_choice = format!("{}-{}{}", menu_char, choice, " ".repeat(pad_length));
                if padded_choice.len() > max_choice_length {
                    padded_choice.truncate(max_choice_length);
                }
                more_lines[current_line_counter as usize].push_str(&format!("{}{}", line_marker, padded_choice));
            }

            let mut lines_too_long = false;
            for line in &more_lines {
                let trimmed_line = line.trim();
                if !trimmed_line.is_empty() && !ran_out_of_vertical_space(
                    line_counter,
                    already_full,
                    terminal_height,
                    choices_len,
                    more_choices_in.len(),
                ) {
                    let effective_line_length = trimmed_line.len();
                    if effective_line_length > 80 { // terminalWidth assumed to be 80
                        lines_too_long = true;
                    }
                    let display_line = if line.len() > 80 {
                        line[0..80].to_string()
                    } else {
                        line.clone()
                    };
                    println!("{}", display_line); // + Color.reset - assuming reset is not needed after truncation
                }
            }
            if lines_too_long {
                println!("(Some lines were longer than the terminal width and have been truncated.)");
            }
        }
    }

    fn display_visual_separator() {
        println!("------------------------------");
    }

    fn lines_left(_leading_text_size: usize, choices_size: usize) -> i32 {
        25 - choices_size as i32 - 2 // Example calculation
    }

    fn get_user_input_char() -> (u8, bool) {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read line");
        if let Some(char) = input.chars().next() {
            (char as u8, false) // Assuming no alternate action for simplicity
        } else {
            (0, false) // Or handle empty input as needed
        }
    }
*/
//%%%%
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
        more_choices_in: Option<Vec<&str>>, /* = None*/
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

    /// Like ask_which but if user makes the alternate action on a choice (eg, double-click, click+differentButton,
    /// right-click, presses "alt+letter"), then it tells you so in the 2nd (boolean) part of the return value.
    pub fn ask_which_choice_or_its_alternate(
        &self,
        leading_text_in: Option<Vec<&str>>,
        choices_in: Vec<&str>,
        more_choices_in: Option<Vec<&str>>, /* = None*/
        include_esc_choice_in: bool, /* = true*/
        trailing_text_in: Option<&str>, /* = None*/
        highlight_index_in: Option<usize>, /* = None*/
        //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) within this method, below!
        secondary_highlight_index_in: Option<usize>, /* = None*/
        default_choice_in: Option<usize>, /* = None*/
    ) -> Option<(usize, bool)> {
        // This attempts to always use as menu option keystroke choices: numbers for "choices" (such as major operations available on the
        // current entity) and letters for "moreChoices" (such as attributes of the current entity to select for further work).  But if
        // there are too many "choices", it will use letters for those as well.
        // I.e., 2nd part of menu ("moreChoices") always starts with a letter, not a #, but the 1st part can use numbers+letters as necessary.
        // This is for the user experience: it seems will be easier to remember how to get around one's own model if attributes always start with
        // 'a' and go from there.
        if choices_in.is_empty() {
            panic!("choices_in must not be empty");
        }

        let max_choice_length = Util::max_name_length();
        
        // Build first menu characters: "123456789"
        let mut first_menu_chars = String::new();
        for number in 1..=9 {
            if number <= choices_in.len() {
                first_menu_chars.push(char::from_digit(number as u32, 10).unwrap());
            }
        }
        
        let possible_menu_chars = format!("{}{}", first_menu_chars, TextUI::MENU_CHARS_LIST);
        
        // Make sure caller didn't send more than the # of things we can handle
        let more_choices_len = more_choices_in.as_ref().map_or(0, |v| v.len());
        if (choices_in.len() + more_choices_len) > possible_menu_chars.len() {
            panic!("Programming error: there are more choices provided ({}) than the menu can handle ({})",
                   choices_in.len() + more_choices_len, possible_menu_chars.len());
        }

        let mut already_full = false;
        let mut line_counter: i32 = 0;
        let mut all_allowed_answers = String::new();
        let mut last_menu_chars_index: i32 = -1;

        let next_menu_char = |last_index: &mut i32, all_answers: &mut String, possible_chars: &str| -> String {
            let next = *last_index + 1;
            *last_index = next;
            if next as usize >= possible_chars.len() {
                return "(ran out)".to_string();
            }
            let next_char = possible_chars.chars().nth(next as usize).unwrap();
            all_answers.push(next_char);
            next_char.to_string()
        };

        let ran_out_of_vertical_space = |line_count: &mut i32, already_full_in: &mut bool| -> bool {
            *line_count += 1;
            if *already_full_in {
                return *already_full_in;
            } else if *line_count > self.terminal_height() {
                // (+ 1 above to leave room for the error message line, below)
                let unshown_count: i32 = 
                    (choices_in.len() + more_choices_len) as i32 - *line_count - 1;
                println!("==============================");
                println!("FYI: Unable to show remaining {} items in the available screen space(!?). Consider code change to pass the \
                        right number of them, relaunching w/ larger terminal, or grouping things?  (ref: {}/{}/{}/{}/{})",
                        unshown_count, already_full_in, line_count, self.terminal_height(), self.terminal_width(), self.get_terminal_class());
                println!("Not going to fail over this, but it might be fixed, especially if you can reproduce it consistently.");
                println!("==============================");
                //already_full = true //not failing after all (setting this to false causes ExpectIt tests to fail when run in IDE)
                *already_full_in = true;
                return true;
            } 
            false
        };

        let show_choices = |line_count: &mut i32, already_full_in: &mut bool| {
            // see containing method description: these choices are 1-based when considered from the human/UI perspective:
            let mut index: usize = 1;

            for choice in &choices_in {
                if !ran_out_of_vertical_space(line_count, already_full_in) {
                    let menu_char = next_menu_char(&mut last_menu_chars_index, &mut all_allowed_answers, &possible_menu_chars);
                    let default_text = if default_choice_in.is_some() && index == default_choice_in.unwrap() {
                        "/Enter"
                    } else {
                        ""
                    };
                    println!("{}{}-{}", menu_char, default_text, choice);
                }
                index += 1;
            }
            if include_esc_choice_in && !ran_out_of_vertical_space(line_count, already_full_in) {
                println!("0/ESC - back/previous menu");
            }
        };

        let show_more_choices = |line_count: &mut i32, already_full_in: &mut bool| {
            if let Some(more_choices) = &more_choices_in {
                if more_choices.is_empty() {
                    return;
                }
                
                // This collection size might be much larger than needed (given multiple columns of display) 
                // but that's better than having more complex calculations
                let mut more_lines: Vec<String> = vec![String::new(); more_choices.len()];
                
                let lines_left_here = self.lines_left(
                    leading_text_in.as_ref().map_or(0, |v| v.len()), 
                    choices_in.len()
                );
                
                let mut line_counter_local = -1;
                // Now build the lines out of columns before displaying them
                let mut index = -1;
                
                for choice in more_choices {
                    index += 1;
                    line_counter_local += 1;
                    
                    if line_counter_local >= lines_left_here as i32 {
                        // 1st is 0-based, 2nd is 1-based
                        line_counter_local = 0; // Wraps to next column
                    }
                    
                    // Not explicitly putting extra space between columns, because space can be in short supply,
                    // and probably some of the choices will be shorter than the max length, to provide enough 
                    // visual alignment/separation anyway. But make them equal length:
                    let line_marker = if highlight_index_in.is_some() && highlight_index_in.unwrap() == index as usize {
                        Color::blue("*")
                    } else if secondary_highlight_index_in.is_some() && secondary_highlight_index_in.unwrap() == index as usize {
                        Color::green("+")
                    } else {
                        " ".to_string()
                    };
                    
                    let pad_length = max_choice_length - choice.len() - TextUI::CHOOSER_MENU_PREFIX_LENGTH - 1;
                    let menu_char = next_menu_char(&mut last_menu_chars_index, &mut all_allowed_answers, &possible_menu_chars);
                    
                    let line = format!("{}{}-{}", line_marker, menu_char, choice);
                    more_lines[line_counter_local as usize] += &line;
                    
                    for _ in 0..pad_length {
                        more_lines[line_counter_local as usize] += " ";
                    }
                }
                
                let mut lines_too_long = false;
                for line in more_lines {
                    if !line.trim().is_empty() && !ran_out_of_vertical_space(line_count, already_full_in) {
                        // Idea for bugfix: adjust the effective_line_length for non-displaying chars
                        // that make up the color of the lineMarker above!
                        let effective_line_length = line.trim().len();
                        
                        if effective_line_length > self.terminal_width() as usize {
                            lines_too_long = true;
                        }
                        
                        // (Appending Color.reset to the string in case it got cut with the substring cmd,
                        // allowing the color to bleed to subsequent lines.)
                        let term_width = self.terminal_width() as usize;
                        let line_to_display = if line.len() > term_width {
                            format!("{}{}", &line[0..term_width], Color::reset())
                        } else {
                            format!("{}{}", line, Color::reset())
                        };
                        
                        println!("{}", line_to_display);
                    }
                }
                
                if lines_too_long {
                    // This could mean that either the terminal width is less than max_name_length characters, or
                    // that there is a bug in the display logic:
                    println!("(Some lines were longer than the terminal width and have been truncated.)");
                }
            }
        };

        self.display_visual_separator();
        
        if let Some(leading_text) = &leading_text_in {
            if !leading_text.is_empty() {
                for prompt in leading_text {
                    line_counter += 1;
                    println!("{}", prompt);
                }
            }
        }
        
        show_choices(&mut line_counter, &mut already_full);
        show_more_choices(&mut line_counter, &mut already_full);
        
        if let Some(trailing_text) = trailing_text_in {
            if !trailing_text.is_empty() {
                println!("{}", trailing_text);
            }
        }

        let (answer, user_chose_alternate) = self.get_user_input_char();
        
        if answer != 27 && answer != b'0' && answer != 13 && !all_allowed_answers.contains(answer as char) {
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
        } else if answer == 13 && (default_choice_in.is_some() || highlight_index_in.is_some()) {
            // User hit Enter i.e. '\r', so take the one that was passed in as default, or highlighted
            if default_choice_in.is_some() {
                Some((default_choice_in.unwrap(), user_chose_alternate))
            } else {
                Some((choices_in.len() + highlight_index_in.unwrap() + 1, user_chose_alternate))
            }
        } else if include_esc_choice_in && (answer == b'0' || answer == 27) {
            None
        } else {
            // Result from this function is 1-based, but 'answer' is 0-based
            let index = possible_menu_chars.find(answer as char).unwrap() + 1;
            Some((index, user_chose_alternate))
        }
    }








    fn is_valid_yes_no_answer(s: &str, _: &TextUI) -> bool {
        s.to_lowercase() == "y"
            || s.to_lowercase() == "yes"
            || s.to_lowercase() == "n"
            || s.to_lowercase() == "no"
    }

    fn is_valid_yes_no_or_blank_answer(s: &str, ui: &TextUI) -> bool {
        Self::is_valid_yes_no_answer(s, ui) || s.trim().is_empty()
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
            /*%%escKeySkipsCriteriaCheck = */ allow_blank_answer,
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
        // if allow_blank_answer && (ans.isEmpty || ans.get.trim.isEmpty) { None }
        // else if !allow_blank_answer && (ans.isEmpty || ans.get.trim.isEmpty) {
        //   throw new OmException("A blank answer was not allowed by the caller but was somehow found here; probably a programming mistake.")
        // }
        // else if ans.isEmpty { throw new OmException("How did we get here?")
        // else if ans.get.toLowerCase.startsWith("y")) Some(true)
        // else Some(false)
    }

    /*
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
/* %%
package org.onemodel.core

import org.onemodel.core.controllers.Controller

import scala.annotation.tailrec

//idea: should go through Controller to get this, so UI layer doesn't have to talk all the way to the model layer? enforce w/ scoping rules?

import org.onemodel.core.model.FileAttribute

import java.io._

import jline.console.{ConsoleReader, KeyMap}
import org.apache.commons.io.FilenameUtils
*/
