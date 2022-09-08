/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004, 2008-2019 inclusive, and 2022, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
// use std::env;
// pub mod controllers;
use crate::controllers::controller::Controller;
// pub mod util;
use crate::util::Util;
use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};

pub struct TextUI {
    //%%read up something in rust book, see examples: better way to handle it? how instantiate w/o all its vars being pub?
    pub testing: bool,
}

impl TextUI {
    //i.e., for the "n-" menu number prefix on each option shown in "askWhich":
    const MENU_CHARS: &'static str =
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
    const CHOOSER_MENU_PREFIX_LENGTH: i32 = 2;

    //%%revu what the book says about things that need to be pub -- everything called fr outside that module?
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

    fn display_text1(&self, text: String) {
        self.display_text2(text, true);
    }
    fn display_text2(&self, text: String, wait_for_keystroke: bool) {
        self.display_text3(text, wait_for_keystroke, None);
    }
    pub fn display_text3(&self, text: String, wait_for_keystroke: bool, pre_prompt: Option<String>) {
        TextUI::display_visual_separator();
        println!("{}", text);

        if wait_for_keystroke && (!self.testing) {
            print!("{}", pre_prompt.unwrap_or(String::from("")));
            println!("Press any key to continue...");
            TextUI::get_user_input_char();
        }
    }

/* %%
    fn initializeTerminal(): jline.Terminal = {
    synchronized {
                   if (mCleanupStarted) null
                   else {
                     let terminal: jline.Terminal = jline.TerminalFactory.get();
                     // parts of the below are from jline's Terminal.setupTerminal(); see the end of the file LICENSE for its license information.
                     //try terminal.initializeTerminal()
                     //catch {
                     //  case e: Exception =>
                     //    e.printStackTrace()
                     //    new jline.UnsupportedTerminal()
                     //}
                     mJlineTerminalInitFinished = true
                     terminal
                   }
                 }
  }

    fn initializeReader(): ConsoleReader = {
    synchronized {
                   if (mCleanupStarted) null
                   else {
                     let is: InputStream = if (inIn.isEmpty) System.in else inIn.get;
                     // the next line would be simpler if we added a method jline.ConsoleReader.setTerminal, or such: the
                     // rest of it is just copied from the simplest constructor in that file.
                     /*val os:OutputStream = new PrintWriter(new OutputStreamWriter(System.out,
                                                                     System.getProperty("jline.WindowsTerminal.output.encoding",
                                                                                        System.getProperty("file.encoding")))*/
                     let jlineReader = new ConsoleReader(is, System.out, mTerminal);
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
                     mJlineReaderInitFinished = true
                     jlineReader
                   }
                 }
  }

  //val cleanup: Thread = new Thread() {
  //  override def run() {
  //    synchronized {
  //                   mCleanupStarted = true
  //                   if (mJlineTerminalInitFinished || mJlineReaderInitFinished) jlineReader.getTerminal.asInstanceOf[jline.UnixTerminal].restoreTerminal()
  //                 }
  //  }
  //}
  //Runtime.getRuntime.addShutdownHook(cleanup)

  let mut out: PrintStream = System.out;

    fn setOutput(out: PrintStream) {
    this.out = out
  }

  //def setInput(in: InputStream) {
  //  jlineReader.setInput(in)
  //}
*/
/* %%$%
  /**
   * The # of items to try to display on the screen at one time.
   */
    fn terminalHeight: Int = {
    mTerminal.getHeight
  }

    fn terminalWidth: Int = {
    if (!Util.isWindows) {
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

    fn get_user_input_char() { //%%-> (char, bool) {
        //%%$%fixme to call next one w/ None as parm, make that one work.
        println!("testinggh1");
        use std::io;
        let mut input=String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                println!("{} bytes input", n);
                println!("input: {}", input);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    //%%make that parm not ignored, or what?:
    // fn get_user_input_char1(/*allowedCharsIn_CURRENTLY_IGNORED: List[Char]*/) -> (char, bool) {
    //     let mut input: i32 = jlineReader.readCharacter(true);
    //     if (!input.isValidChar) {
    //       throw new Exception("Unexpected non-char value " + input + " from readCharacter().")
    //     }
    //     let mut isAltKeyCombo: bool = false;
    //     if (input > 1000) {
    //       // this means that the user pressed an alt-key combination (i.e.: my kludge in my modified copy of jline2's
    //       // readCharacter(boolean checkForAltKeyCombo) has been invoked.
    //       isAltKeyCombo = true
    //       input -= 1000
    //     }
    //
    //     // show user what they just did
    //     if (input == 27) println("ESC")
    //     else println(new String(Array(input.toChar)))
    //     (input.toChar, isAltKeyCombo)
    // }

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

/* %%
  /* Returns the string entered (None if the user just wants out of this question or whatever, unless escKeySkipsCriteriaCheck is false).
   * The parameter "criteria"'s Option is a function which takes a String (which will be the user input), which it checks for validity.
   * If the entry didn't meet the criteria, it repeats the question until it does or user gets out w/ ESC.
   * A simple way to let the user know why it didn't meet the criteria is to put them in the leading text.
   */
  //@tailrec //see below note on 'recursive' for why removed 4 now.
  final def askForString(leadingTextIn: Option[Array[String]],
                         criteriaIn: Option[(String) => Boolean] = None,
                         defaultValueIn: Option[String] = None,
                         isPasswordIn: Boolean = false,
                         //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) within this method, below!
                         escKeySkipsCriteriaCheck: Boolean = true): Option[String] = {
    let mut count = 0;
    let lastLineOfPrompt: String = {;
      let mut lastLineOfPrompt = "";
      if (leadingTextIn.isDefined) {
        for (prompt <- leadingTextIn.get) {
          count = count + 1
          if (count < leadingTextIn.get.length) {
            // all but the last one
            println(prompt)
          } else {
            //print(prompt)
            //if (inDefaultValue.isDefined && inDefaultValue.get.length() > 0) {
            //  print(" [defaults to " + inDefaultValue.get + "]")
            //}
            //println(": ")
            lastLineOfPrompt = prompt + ": "
          }
        }
      }
      lastLineOfPrompt
    }
    // idea: make this better by using features of or tweaking jline2? Or...? But at least make it easy to see when out of room!
    //val promptToShowStringSizeLimit = "(Max name length is  " + Controller.maxNameLength
    let endPrompt = "(... which ends here: |)";
    if (lastLineOfPrompt.length > 1 && lastLineOfPrompt.length + endPrompt.length - 1 <= Util.maxNameLength) {
      let spaces: StringBuilder = new StringBuilder("");
      // (the + 1 in next line is for the closing parenthesis in the prompt, which comes after the visual end position marker
      let padLength: i32 = Util.maxNameLength - lastLineOfPrompt.length - endPrompt.length + 1;
      for (x <- 0 until padLength) {
        spaces.append(" ")
      }
      println(lastLineOfPrompt + spaces + endPrompt)
    } else println(lastLineOfPrompt)

    // thread is for causing jline to display the default text for editing, after readLine call begins.  (is there a better way?)
    new Thread {
      override def run() {
        // wait for the readline below to start, before putting something in it
        Thread.sleep(80)
        jlineReader.putString(defaultValueIn.getOrElse(""))
        jlineReader.redrawLine()
        jlineReader.flush()
      }
    }.start()

    //jlineReader.readLine()
    let line = jlineReader.readLine(null, if (isPasswordIn) '*' else null);
    if (line == null) {
      None
    } else {
      def checkCriteria(line: String): Option[String] = {
        if (criteriaIn.isEmpty || criteriaIn.get(line)) {
          Some(line)
        } else {
          display_text("Didn't pass the criteria; please re-enter.")
          // this gets "recursive call not in tail position", until new version of jvm that allows scala2do it?
          askForString(leadingTextIn, criteriaIn, defaultValueIn, isPasswordIn, escKeySkipsCriteriaCheck)
        }
      }

      if (line.isEmpty && escKeySkipsCriteriaCheck) {
        None
      } else {
        checkCriteria(line)
      }
    }
  }

    fn linesLeft(numOfLeadingTextLinesIn: Int, numChoicesAboveColumnsIn: Int): Int = {
    let linesUsedBeforeMoreChoices = numOfLeadingTextLinesIn + numChoicesAboveColumnsIn + 5 // 5 as described in one caller;
    terminalHeight - linesUsedBeforeMoreChoices
  }

  /** The # of attributes ("moreChoices" elsewhere) that will likely fit in the space available on the
    * screen AFTER the preceding leadingText lines + menu size + 5: 1 line added by askWhich(...) (for the 0/ESC menu option), 1 line for the visual separator,
    * and 1 line for the cursor at the bottom to not push things off the top, and 2 more because entity/group names and the line that shows them at the
    * top of a menu are long & wrap, so they were still pushing things off the top of the visual space (could have made it 3 more for small windows, but that
    * might make the list of data too short in some cases, and 2 is probably usually enough if windows aren't too narrow).
    * based on # of available columns and a possible max column width.
    * SEE ALSO the method linesLeft, which actually has/uses the number.
    */
    fn maxColumnarChoicesToDisplayAfter(numOfLeadingTextLinesIn: Int, numChoicesAboveColumnsIn: Int, fieldWidthIn: Int): Int = {
    let maxMoreChoicesBySpaceAvailable = linesLeft(numOfLeadingTextLinesIn, numChoicesAboveColumnsIn) * columnsPossible(fieldWidthIn + CHOOSER_MENU_PREFIX_LENGTH);
    // the next 2 lines are in coordination with a 'require' statement in askWhich, so we don't fail it:
    let maxMoreChoicesByMenuCharsAvailable = TextUI.menuCharsList.length;
    math.min(maxMoreChoicesBySpaceAvailable, maxMoreChoicesByMenuCharsAvailable)
  }

    fn columnsPossible(columnWidthIn: Int): Int = {
    require(columnWidthIn > 0)
    // allow at least 1 column, even with a smaller terminal width
    math.max(terminalWidth / columnWidthIn, 1)
  }

  /** The parm "choices" are shown in a single-column list; the "moreChoices" are shown in columns as space allows.
    *
    * The return value is either None (if user just wants out), or Some(the # of the result chosen) (1-based, where the index is
    * against the *combined* choices and moreChoices).  Ex., if the choices parameter has 3 elements, and moreChoices has 5, the
    * return value can range from 1-8 (1-based, not 0-based!).
    *
    * If calling methods are kept small, it should be easy for them to visually determine which 'choice's go with the return value;
    * see current callers for examples of how to easily determine which 'moreChoice's go with the return value.
    *
    * @param highlightIndexIn 0-based (like almost everything; exceptions are noted.).
    * @param secondaryHighlightIndexIn 0-based.
    * @param defaultChoiceIn 1-based.
    *
    * @return 1-based (see description).
    *
    */
  final def askWhich(leadingTextIn: Option[Array[String]],
                     choicesIn: Array[String],
                     moreChoicesIn: Array[String] = Array(),
                     includeEscChoiceIn: Boolean = true,
                     trailingTextIn: Option[String] = None,
                     highlightIndexIn: Option[Int] = None,
                     secondaryHighlightIndexIn: Option[Int] = None,
                     defaultChoiceIn: Option[Int] = None): Option[Int] = {
    let result = askWhichChoiceOrItsAlternate(leadingTextIn, choicesIn, moreChoicesIn, includeEscChoiceIn, trailingTextIn,;
                                              highlightIndexIn, secondaryHighlightIndexIn, defaultChoiceIn)
    if (result.isEmpty) None
    else Some(result.get._1)
  }

  /** Like askWhich but if user makes the alternate action on a choice (eg, double-click, click+differentButton, right-click, presses "alt+letter"),
    * then it tells you so in the 2nd (boolean) part of the return value.
    * */
  @tailrec
  final def askWhichChoiceOrItsAlternate(leadingTextIn: Option[Array[String]],
                     choicesIn: Array[String],
                     moreChoicesIn: Array[String] = Array(),
                     includeEscChoiceIn: Boolean = true,
                     trailingTextIn: Option[String] = None,
                     highlightIndexIn: Option[Int] = None,
                     //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) within this method, below!
                     secondaryHighlightIndexIn: Option[Int] = None,
                     defaultChoiceIn: Option[Int] = None): Option[(Int, Boolean)] = {
    // This attempts to always use as menu option keystroke choices: numbers for "choices" (such as major operations available on the
    // current entity) and letters for "moreChoices" (such as attributes of the current entity to select for further work).  But if
    // there are too many "choices", it will use letters for those as well.
    // I.e., 2nd part of menu ("moreChoices") always starts with a letter, not a #, but the 1st part can use numbers+letters as necessary.
    // This is for the user experience: it seems will be easier to remember how to get around one's own model if attributes always start with
    // 'a' and go from there.
    require(choicesIn.length > 0)

    let maxChoiceLength = Util.maxNameLength;

    let firstMenuChars: StringBuffer = {;
      //up to: "123456789"
      let chars = new StringBuffer;
      for (number: Int <- 1 to 9) if (number <= choicesIn.length) {
        chars.append(number)
      }
      chars
    }
    let possibleMenuChars = firstMenuChars + TextUI.menuCharsList;
    // make sure caller didn't send more than the # of things we can handle
    require((choicesIn.length + moreChoicesIn.length) <= possibleMenuChars.length, "Programming error: there are more choices provided (" +
                                                                               (choicesIn.length + moreChoicesIn.length) + ") than the menu can handle (" +
                                                                               possibleMenuChars.length + ")")

    let alreadyFull = false;
    let mut lineCounter: i32 = 0;
    let allAllowedAnswers = new StringBuffer;

    let mut lastMenuCharsIndex: i32 = -1;
    def nextMenuChar(): String = {
      let next = lastMenuCharsIndex + 1;
      lastMenuCharsIndex = next
      if (next > possibleMenuChars.length) {
        return "(ran out)"
      }
      allAllowedAnswers.append(possibleMenuChars.charAt(next))
      new String("" + possibleMenuChars.charAt(next))
    }

    def ranOutOfVerticalSpace(): Boolean = {
      lineCounter = lineCounter + 1
      if (alreadyFull) {
        alreadyFull
      } else if ((!alreadyFull) && lineCounter > terminalHeight) {
        // (+ 1 above to leave room for the error message line, below)
        let unshownCount: i32 = choicesIn.length + moreChoicesIn.length - lineCounter - 1;
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

    def showChoices() {
      // see containing method description: these choices are 1-based when considered from the human/UI perspective:
      let mut index: i32 = 1;

      for (choice <- choicesIn) {
        if (!ranOutOfVerticalSpace) {
          println(nextMenuChar() +
                  (if (defaultChoiceIn.isDefined && index == defaultChoiceIn.get) "/Enter" else "") +
                  "-" + choice)
        }
        index += 1
      }
      if (includeEscChoiceIn && !ranOutOfVerticalSpace) {
        println("0/ESC - back/previous menu")
      }
    }

    def showMoreChoices() {
      if (moreChoicesIn.length == 0) {
        //noinspection ScalaUselessExpression (intentional style violation, for readability):
        Unit
      } else {
        // this collection size might be much larger than needed (given multiple columns of display) but that's better than having more complex calculations.
        let moreLines = new Array[StringBuffer](moreChoicesIn.length);
        for (i <- moreLines.indices) {
          moreLines(i) = new StringBuffer()
        }
        let linesLeftHere = linesLeft(leadingTextIn.size, choicesIn.length);
        let mut lineCounter = -1;
        // now build the lines out of columns be4 displaying them.
        let mut index = -1;
        for (choice <- moreChoicesIn) {
          index += 1
          lineCounter = lineCounter + 1
          if (lineCounter >= linesLeftHere) {
            // 1st is 0-based, 2nd is 1-based
            lineCounter = 0 //wraps to next column
          }
          // Not explicitly putting extra space between columns, because space can be in short supply, and probably some of the choices
          // will be shorter than the max length, to provide enough visual alignment/separation anyway.  But make them equal length:
          let lineMarker: String =;
            if (highlightIndexIn.getOrElse(None) == index) Color.blue("*")
            else if (secondaryHighlightIndexIn.getOrElse(None) == index) Color.green("+")
            else " "
          let padLength = maxChoiceLength - choice.length - CHOOSER_MENU_PREFIX_LENGTH - 1;
          moreLines(lineCounter).append(lineMarker + nextMenuChar() + "-" + choice)
          for (x <- 0 until padLength) {
            moreLines(lineCounter).append(" ")
          }
        }
        let mut linesTooLong = false;
        for (line <- moreLines) {
          if (line.toString.trim.length > 0 && !ranOutOfVerticalSpace) {
            // idea for bugfix: adjust the effectiveLineLength for non-displaying chars that make up the color of the lineMarker above!
            let effectiveLineLength = line.toString.trim.length;
            if (effectiveLineLength > terminalWidth) {
              linesTooLong = true
            }
            // (Appending Color.reset to the string in case it got cut with the substring cmd, allowing the color to bleed to subsequent lines.)
            println(line.toString.substring(0, math.min(line.length, terminalWidth)) + Color.reset)
          }
        }
        if (linesTooLong) {
          // This could mean that either the terminal width is less than " + Controller.maxNameLength + " characters, or
          // that there is a bug in the display logic:
          println("(Some lines were longer than the terminal width and have been truncated.)")
        }
      }
    }


    display_visual_separator()
    if (leadingTextIn.isDefined && leadingTextIn.get.length > 0) {
      for (prompt <- leadingTextIn.get) {
        lineCounter = lineCounter + 1
        println(prompt)
      }
    }
    showChoices()
    showMoreChoices()
    if (trailingTextIn.isDefined && trailingTextIn.get.nonEmpty) println(trailingTextIn.get)

    let (answer: Char, userChoseAlternate: Boolean) = get_user_input_char;
    if (answer != 27 && answer != '0' && answer != 13 && (!allAllowedAnswers.toString.contains(answer.toChar))) {
      println("unknown choice: " + answer)
      askWhichChoiceOrItsAlternate(leadingTextIn, choicesIn, moreChoicesIn, includeEscChoiceIn, trailingTextIn, highlightIndexIn, secondaryHighlightIndexIn,
                                   defaultChoiceIn)
    } else if (answer == 13 && (defaultChoiceIn.isDefined || highlightIndexIn.isDefined)) {
      // user hit Enter ie '\r', so take the one that was passed in as default, or highlighted
      if (defaultChoiceIn.isDefined) {
        Some(defaultChoiceIn.get, userChoseAlternate)
      } else {
        Some(choicesIn.length + highlightIndexIn.get + 1, userChoseAlternate)
      }
    } else if (includeEscChoiceIn && (answer == '0' || answer == 27)) {
      None
    } else {
      Some(possibleMenuChars.indexOf(answer) + 1, userChoseAlternate) // result from this function is 1-based, but 'answer' is 0-based.
    }
  }

    fn isValidYesNoAnswer(s: String): Boolean = {
    s.toLowerCase == "y" ||
    s.toLowerCase == "yes" ||
    s.toLowerCase == "n" ||
    s.toLowerCase == "no"
  }

    fn isValidYesNoOrBlankAnswer(s: String): Boolean = {
    isValidYesNoAnswer(s) ||
    s.trim.isEmpty
  }

  /** true means yes, None means user wants out. */
    fn askYesNoQuestion(promptIn: String, defaultValueIn: Option[String] = Some("n"), allowBlankAnswer: Boolean = false): Option[Boolean] = {
    let ans = askForString(Some(Array[String](promptIn + " (y/n)")),;
                           if (allowBlankAnswer) Some(isValidYesNoOrBlankAnswer) else Some(isValidYesNoAnswer),
                           defaultValueIn, escKeySkipsCriteriaCheck = allowBlankAnswer)
    if (allowBlankAnswer && (ans.isEmpty || ans.get.trim.isEmpty)) None
    else if (!allowBlankAnswer && (ans.isEmpty || ans.get.trim.isEmpty)) {
      throw new OmException("A blank answer was not allowed by the caller but was somehow found here; probably a programming mistake.")
    }
    else if (ans.isEmpty) throw new OmException("How did we get here?")
    else if (ans.get.toLowerCase.startsWith("y")) Some(true)
    else Some(false)
  }

  /** This is in the UI code because probably a GUI would do it very differently.
    */
    fn getExportDestination(originalPathIn: String, originalMd5HashIn: String): Option[File] = {
    def newLocation(originalNameIn: String): Option[File] = {
      let oldNameInTmpDir: File = new File(System.getProperty("java.io.tmpdir"), originalNameIn);
      if (oldNameInTmpDir.getParentFile.canWrite && !oldNameInTmpDir.exists()) Some(oldNameInTmpDir)
      else {
        let (baseName, extension) = Util.getUsableFilename(originalPathIn);
        Some(File.createTempFile(baseName + "-", extension))
      }
    }
    let originalFile = new File(originalPathIn);
    let originalContainingDirectory = originalFile.getParentFile;
    let originalName = FilenameUtils.getBaseName(originalPathIn);
    let baseConfirmationMessage = "Put the file in the original location: \"" + originalPathIn + "\"";
    let restOfConfirmationMessage = " (No means put it in a new/temporary location instead.)";
    if (originalContainingDirectory != null && originalContainingDirectory.exists && (!originalFile.exists)) {
      let ans = askYesNoQuestion(baseConfirmationMessage + "?" + restOfConfirmationMessage);
      if (ans.isEmpty) None
      else {
        if (ans.get) Some(originalFile)
        else newLocation(originalName)
      }
    } else {
      let yesExportTheFile: Option[Boolean] = {;
        if (originalFile.exists) {
          if (FileAttribute.md5Hash(originalFile) != originalMd5HashIn) Some(true)
          else {
            askYesNoQuestion("The file currently at " + originalPathIn + " is identical to the one stored.  Export anyway?  (Answering " +
                             "'y' will still allow choosing whether to overwrite it or write to a new location instead.)")
          }
        } else Some(true)
      }
      if (yesExportTheFile.isEmpty || !yesExportTheFile.get) None
      else {
        if (originalContainingDirectory != null && !originalContainingDirectory.exists) {
          newLocation(originalName)
        } else {
          require(originalFile.exists, "Logic got here unexpectedly, to a point where the original file is expected to exist but does not: " + originalPathIn)
          let ans = askYesNoQuestion(baseConfirmationMessage + "\" (overwriting the current copy)?" + restOfConfirmationMessage);
          if (ans.isEmpty) None
          else if (ans.get) Some(originalFile)
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
