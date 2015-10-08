/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2015 inclusive, Luke A Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel

import org.onemodel.controller.Controller

import scala.annotation.tailrec

//idea: should go through controller to get this, so UI layer doesn't have to talk all the way to the model layer? enforce w/ scoping rules?

import org.onemodel.model.FileAttribute

//import scala.runtime.RichChar

import java.io._

import jline.console.{ConsoleReader, KeyMap}
import org.apache.commons.io.FilenameUtils

/** Provides a text-based interface for efficiency, or for people who like that,
  * The first OM user interface, it is intended to demonstrate basic concepts until we can make something more friendly.
  *
  * Improvements to this class should START WITH MAKING IT BETTER TESTED, then changing vars to vals?, None for null, delaying side effects more,
  * shorter methods, other better scala style...
  */

object TextUI {
  val NEWLN: String = System.getProperty("line.separator")
  val menuCharsList: String = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"

  def main(args: Array[String]) {
    new TextUI(args).launchUI()
  }
}

class TextUI(args: Array[String] = Array[String](), val inIn: Option[InputStream] = None) {
  //i.e., for the "n-" menu number prefix on each option shown in "askWhich":
  val objectChooserMenuPrefixLength: Int = 2
  val forceUsernamePasswordPrompt = if (args.length == 1) true else false
  val (username, password): (Option[String], Option[String]) = if (args.length == 2) (Some(args(0)), Some(args(1))) else (None, None)

  // (making some lazy vals instead of vars because it's considered generally cleaner to use vals, and lazy in case they are not
  // needed for unit tests)
  lazy val controller: Controller = new Controller(this, forceUsernamePasswordPrompt, username, password)
  lazy val mTerminal: jline.Terminal = initializeTerminal()
  val jlineReader: ConsoleReader = initializeReader()

  // used to coordinate the mTerminal initialization (problems still happened when it wasn't lazy), and the cleanup thread, so that
  // the cleanup actually happens.
  private val mCleanupStarted: Boolean = false

  private var mJlineTerminalInitFinished: Boolean = false
  private var mJlineReaderInitFinished: Boolean = false

  def initializeTerminal(): jline.Terminal = {
    synchronized {
                   if (mCleanupStarted) null
                   else {
                     val terminal: jline.Terminal = jline.TerminalFactory.get()
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

  def initializeReader(): ConsoleReader = {
    synchronized {
                   if (mCleanupStarted) null
                   else {
                     val is: InputStream = if (inIn.isEmpty) System.in else inIn.get
                     // the next line would be simpler if we added a method jline.ConsoleReader.setTerminal, or such: the
                     // rest of it is just copied from the simplest constructor in that file.
                     /*val os:OutputStream = new PrintWriter(new OutputStreamWriter(System.out,
                                                                     System.getProperty("jline.WindowsTerminal.output.encoding",
                                                                                        System.getProperty("file.encoding")))*/
                     val jlineReader = new ConsoleReader(is, System.out, mTerminal)
                     jlineReader.setBellEnabled(false)
                     //handy sometimes:
                     //jlineReader.setDebug(new PrintWriter(System.err))

                     // allow ESC to abort an editing session (in combination w/ jline2 version / modifications):
                     val startingKeyMap: String = jlineReader.getKeyMap
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

  var out: PrintStream = System.out

  def setOutput(out: PrintStream) {
    this.out = out
  }

  //def setInput(in: InputStream) {
  //  jlineReader.setInput(in)
  //}

  /**
   * The # of items to try to display on the screen at one time.
   */
  private def terminalHeight: Int = {
    mTerminal.getHeight
  }

  private def terminalWidth: Int = {
    mTerminal.getWidth
  }

  def getUserInputChar: (Char, Boolean) = {
      getUserInputChar(Nil)
  }

  def getUserInputChar(allowedCharsIn: List[Char]): (Char, Boolean) = {
    var input: Int = jlineReader.readCharacter(true)
    if (!input.isValidChar) {
      throw new Exception("Unexpected non-char value " + input + " from readCharacter().")
    }
    var isAltKeyCombo: Boolean = false
    if (input > 1000) {
      // this means that the user pressed an alt-key combination (i.e.: my kludge in my modified copy of jline2's
      // readCharacter(boolean checkForAltKeyCombo) has been invoked.
      isAltKeyCombo = true
      input -= 1000
    }

    // show user what they just did
    if (input == 27) println("ESC")
    else println(new String(Array(input.toChar)))
    (input.toChar, isAltKeyCombo)
  }

  /** Allows customizing the output stream, for tests.
    */
  def println() {
    out.println()
  }

  def println(s: String) {
    out.println(s)
  }

  private def displayVisualSeparator() {
    for (x <- 1 to 6) println()
    println("==============================================")
  }

  var mTesting = false

  //idea: change this to ".apply"
  def weAreTesting(testing: Boolean) {
    mTesting = testing
  }

  def weAreTesting: Boolean = {
    mTesting
  }

  def displayText(text: String, waitForKeystroke: Boolean = true, prePrompt: Option[String] = None) {
    displayVisualSeparator()
    println(text)

    if (waitForKeystroke && (!weAreTesting)) {
      print(prePrompt.getOrElse(""))
      println("Press any key to continue...")
      getUserInputChar
    }
  }

  def launchUI() {
    controller.start()
  }

  /* Returns the string entered (None if the user just wants out of this question or whatever, OR the default value in that case if one is provided. Is that
     true, or does it  always currently return just an empty string? Looks so. See internal comments for possibly fixing that.)
   * The parameter "criteria"'s Option is a function which takes a String (which will be the user input), which it checks for validity.
   * If the entry didn't meet the criteria, it repeats the question until it does or user gets out w/ ESC.
   * A simple way to let the user know why it didn't meet the criteria is to put them in the leading text.
   */
  //@tailrec //see below note on 'recursive' for why removed 4 now.
  final def askForString(inLeadingText: Option[Array[String]],
                         inCriteria: Option[(String) => Boolean] = None,
                         inDefaultValue: Option[String] = None,
                         inIsPassword: Boolean = false): Option[String] = {
    var count = 0
    val lastLineOfPrompt: String = {
      var lastLineOfPrompt = ""
      if (inLeadingText.isDefined) {
        for (prompt <- inLeadingText.get) {
          count = count + 1
          if (count < inLeadingText.get.length) {
            // all but the last one
            println(prompt)
          }
          else {
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
    //val promptToShowStringSizeLimit = "(Max name length is  " + controller.maxNameLength
    val endPrompt = "(... which ends here: |)"
    if (lastLineOfPrompt.length > 1 && lastLineOfPrompt.length + endPrompt.length - 1 <= controller.maxNameLength) {
      val spaces: StringBuilder = new StringBuilder("")
      // (the + 1 in next line is for the closing parenthesis in the prompt, which comes after the visual end position marker
      val padLength: Int = controller.maxNameLength - lastLineOfPrompt.length - endPrompt.length + 1
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
        jlineReader.putString(inDefaultValue.getOrElse(""))
        jlineReader.redrawLine()
        jlineReader.flush()
      }
    }.start()

    //jlineReader.readLine()
    val line = jlineReader.readLine(null, if (inIsPassword) '*' else null)
    if (line == null) {
      None
    }
    else {
      def checkCriteria(line: String): Option[String] = {
        if (inCriteria.isEmpty || inCriteria.get(line)) {
          Some(line)
        }
        else {
          displayText("Didn't pass the criteria; please re-enter.")
          // this gets "recursive call not in tail position", until new version of jvm that allows scala2do it?
          askForString(inLeadingText, inCriteria, inDefaultValue, inIsPassword)
        }
      }

      if (line.length() == 0 && inDefaultValue.isDefined) {
        // idea: we are currently taking the default value even if there are criteria and it fails: That could be reconsidered.
        // If this is changed, places calling it that rely on its  behavior should be re-evaluated (ie, what is the behavior & how should callers use that?
        // Especially for Controller.askForAttributeValidAndObservedDates, which currently, unfortunately, relies on this mis-behavior, and change the method
        // comment describing its contract TO CORRECTLY DESCRIBE THE MEANING & OPERATION!
        // Maybe the callers should control it via whether there is a default value, & change the cmt of this method, or, they should act based on whether the
        // returned info is blank.  Does this mean also that we should add tests for the text UI, now?
        checkCriteria(inDefaultValue.get)
      }
      else {
        checkCriteria(line)
      }
    }
  }

  private def linesLeft(numOfLeadingTextLines: Int, numChoicesAboveColumns: Int): Int = {
    val linesUsedBeforeMoreChoices = numOfLeadingTextLines + numChoicesAboveColumns + 3 // 3 as described in one caller
    terminalHeight - linesUsedBeforeMoreChoices
  }

  /** The # of attributes ("moreChoices" elsewhere) that will likely fit in the space available on the
    * screen AFTER the preceding leadingText lines + menu size + 3: 1 line added by askWhich(...) (for the 0/ESC menu option), 1 line for the visual separator,
    * and 1 line for the cursor at the bottom to not push things off the top.
    * based on # of available columns and a possible max column width.
    */
  def maxColumnarChoicesToDisplayAfter(numOfLeadingTextLines: Int, numChoicesAboveColumns: Int, fieldWidth: Int): Int = {
    val maxMoreChoicesBySpaceAvailable = linesLeft(numOfLeadingTextLines, numChoicesAboveColumns) * columnsPossible(fieldWidth + objectChooserMenuPrefixLength)
    // the next 2 lines are in coordination with a 'require' statement in askWhich, so we don't fail it:
    val maxMoreChoicesByMenuCharsAvailable = TextUI.menuCharsList.length
    math.min(maxMoreChoicesBySpaceAvailable, maxMoreChoicesByMenuCharsAvailable)
  }

  def columnsPossible(columnWidth: Int): Int = {
    require(columnWidth > 0)
    // allow at least 1 column, even with a smaller terminal width
    math.max(terminalWidth / columnWidth, 1)
  }

  /** The parm "choices" are shown in a single-column list; the "moreChoices" are shown in columns as space allows.
    *
    * The return value is either None (if user just wants out), or Some(the # of the result chosen) (1-based, where the index is
    * against the *combined* choices and moreChoices).  E.g., if the choices parameter has 3 elements, and moreChoices has 5, the
    * return value can range from 1-8 (1-based, not 0-based!).
    *
    * If calling methods are kept small, it should be easy for them to visually determine which 'choice's go with the return value;
    * see current callers for examples of how to easily determine which 'moreChoice's go with the return value.
    */
  final def askWhich(leadingText: Option[Array[String]],
                     choices: Array[String],
                     moreChoices: Array[String] = Array(),
                     includeEscChoice: Boolean = true,
                     trailingText: Option[String] = None,
                     highlightIndexIn: Option[Int] = None,
                     secondaryHighlightIndexIn: Option[Int] = None): Option[Int] = {
    val result = askWhichChoiceOrItsAlternate(leadingText, choices, moreChoices, includeEscChoice, trailingText, highlightIndexIn, secondaryHighlightIndexIn)
    if (result.isEmpty) None
    else Some(result.get._1)
  }

  /** Like askWhich but if user makes the alternate action on a choice (eg, double-click, click+differentButton, right-click, presses "alt+letter"),
    * then it tells you so in the 2nd (boolean) part of the return value. */
  @tailrec
  final def askWhichChoiceOrItsAlternate(leadingText: Option[Array[String]],
                     choices: Array[String],
                     moreChoices: Array[String] = Array(),
                     includeEscChoice: Boolean = true,
                     trailingText: Option[String] = None,
                     highlightIndexIn: Option[Int] = None,
                     secondaryHighlightIndexIn: Option[Int] = None): Option[(Int, Boolean)] = {
    // This attempts to always use as menu option keystroke choices: numbers for "choices" (such as major operations available on the
    // current entity) and letters for "moreChoices" (such as attributes of the current entity to select for further work).  But if
    // there are too many "choices", it will use letters for those as well.
    // I.e., 2nd part of menu ("moreChoices") always starts with a letter, not a #, but the 1st part can use numbers+letters as necessary.
    // This is for the user experience: it seems will be easier to remember how to get around one's own model if attributes always start with
    // 'a' and go from there.
    require(choices.length > 0)

    val maxChoiceLength = controller.maxNameLength

    val firstMenuChars: StringBuffer = {
      //up to: "123456789"
      val chars = new StringBuffer
      for (number: Int <- 1 to 9) if (number <= choices.length) {
        chars.append(number)
      }
      chars
    }
    val possibleMenuChars = firstMenuChars + TextUI.menuCharsList
    // make sure caller didn't send more than the # of things we can handle
    require((choices.length + moreChoices.length) <= possibleMenuChars.length)

    var alreadyFull = false
    var lineCounter: Int = 0
    val allAllowedAnswers = new StringBuffer

    var lastMenuCharsIndex: Int = -1
    def nextMenuChar(): String = {
      val next = lastMenuCharsIndex + 1
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
      }
      else if ((!alreadyFull) && lineCounter > terminalHeight) {
        // (+ 1 above to leave room for the error message line, below)
        val unshownCount: Int = choices.length + moreChoices.length - lineCounter - 1
        println("Unable to show remaining " + unshownCount + " items in the available screen space(!?). Consider code change to pass the " +
                "right number of them, relaunching w/ larger terminal, or grouping things?")
        alreadyFull = true
        alreadyFull
      }
      else false
    }

    def showChoices() {
      for (choice <- choices) {
        if (!ranOutOfVerticalSpace) {
          println(nextMenuChar() + "-" + choice)
        }
      }
      if (includeEscChoice && !ranOutOfVerticalSpace) {
        println("0/ESC - back/previous menu")
      }
    }

    def showMoreChoices() {
      if (moreChoices.length == 0) {
        // (intentional style violation, for readability):
        Unit
      }
      else {
        // this collection size might be much larger than needed (given multiple columns of display) but that's better than having more complex calculations.
        val moreLines = new Array[StringBuffer](moreChoices.length)
        for (i <- moreLines.indices) {
          moreLines(i) = new StringBuffer()
        }
        val linesLeftHere = linesLeft(leadingText.size, choices.length)
        var lineCounter = -1
        // now build the lines out of columns be4 displaying them.
        var index = -1
        for (choice <- moreChoices) {
          index += 1
          lineCounter = lineCounter + 1
          if (lineCounter >= linesLeftHere) {
            // 1st is 0-based, 2nd is 1-based
            lineCounter = 0 //wraps to next column
          }
          // Not explicitly putting extra space between columns, because space can be in short supply, and probably some of the choices
          // will be shorter than the max length, to provide enough visual alignment/separation anyway.  But make them equal length:
          val lineMarker: String =
            if (highlightIndexIn.getOrElse(None) == index) Color.blue("*")
            else if (secondaryHighlightIndexIn.getOrElse(None) == index) Color.green("+")
            else " "
          val padLength = maxChoiceLength - choice.length - objectChooserMenuPrefixLength - 1
          moreLines(lineCounter).append(lineMarker + nextMenuChar() + "-" + choice)
          for (x <- 0 until padLength) {
            moreLines(lineCounter).append(" ")
          }
        }
        var linesTooLong = false
        for (line <- moreLines) {
          if (line.toString.trim.length > 0 && !ranOutOfVerticalSpace) {
            // idea for bugfix: adjust the effectiveLineLength for non-displaying chars that make up the color of the lineMarker above!
            val effectiveLineLength = line.toString.trim.length
            if (effectiveLineLength > terminalWidth) {
              linesTooLong = true
            }
            println(line.toString.substring(0, math.min(line.length, terminalWidth)))
          }
        }
        if (linesTooLong) {
          println("(Some lines were longer than the terminal width and have been truncated.  That could mean that either your terminal width is less than  "
                  + controller.maxNameLength + " characters, or that there is a bug in the display logic.)")
        }
      }
    }

    displayVisualSeparator()
    if (leadingText.isDefined && leadingText.get.length > 0) {
      for (prompt <- leadingText.get) {
        lineCounter = lineCounter + 1
        println(prompt)
      }
    }
    showChoices()
    showMoreChoices()
    if (trailingText.isDefined && trailingText.get.nonEmpty) println(trailingText.get)

    val allowedInputChars: Array[Char] = new Array(allAllowedAnswers.length + 2)
    allowedInputChars(0) = '0'
    allowedInputChars(1) = 27.asInstanceOf[Char] //ESC
    allAllowedAnswers.getChars(0, allAllowedAnswers.length, allowedInputChars, 2)
    val (answer: Char, userChoseAlternate: Boolean) = getUserInputChar(allowedInputChars.toList)

    if (answer != 27 && answer != '0' && (!allAllowedAnswers.toString.contains(answer.toChar))) {
      println("unknown choice: " + answer)
      askWhichChoiceOrItsAlternate(leadingText, choices, moreChoices, includeEscChoice, trailingText)
    }
    else if (includeEscChoice && (answer == '0' || answer == 27)) {
      None
    }
    else {
      Some(possibleMenuChars.indexOf(answer) + 1, userChoseAlternate) // result from this function is 1-based, but 'answer' is 0-based.
    }
  }

  private def isValidYesNoAnswer(s: String): Boolean = {
    s.toLowerCase == "y" ||
    s.toLowerCase == "yes" ||
    s.toLowerCase == "n" ||
    s.toLowerCase == "no"
  }

  private def isValidYesNoOrBlankAnswer(s: String): Boolean = {
    isValidYesNoAnswer(s) ||
    s.trim.isEmpty
  }

  /** true means yes, None means user wants out. */
  def askYesNoQuestion(promptIn: String, defaultValueIn: Option[String] = Some("n"), allowBlankAnswer: Boolean = false): Option[Boolean] = {
    val ans = askForString(Some(Array[String](promptIn + " (y/n)")),
                           if (allowBlankAnswer) Some(isValidYesNoOrBlankAnswer) else Some(isValidYesNoAnswer),
                           defaultValueIn)
    if (ans.isEmpty) None
    else if (allowBlankAnswer && ans.get.trim.isEmpty) None
    else if (ans.get.toLowerCase.startsWith("y")) Some(true)
    else Some(false)
  }

  /** This is in the UI code because probably a GUI would do it very differently.
    */
  def getExportDestinationFile(originalPathIn: String, originalMd5HashIn: String): Option[File] = {
    val origPathFile = new File(originalPathIn)
    val containingDirectory = origPathFile.getParentFile
    if ((!origPathFile.exists) && containingDirectory != null && containingDirectory.exists) Some(origPathFile)
    else {
      val yesExportTheFile: Option[Boolean] = {
        if (origPathFile.exists) {
          if (FileAttribute.md5Hash(origPathFile) != originalMd5HashIn) Some(true)
          else {
            askYesNoQuestion("The file currently at " + originalPathIn + " is identical to the one stored.  Export anyway?  (Answering " +
                             "'y' will still allow choosing whether to overwrite it.)")
          }
        } else Some(true)
      }
      if (yesExportTheFile.isEmpty || !yesExportTheFile.get) None
      else {
        def newLocation(originalNameIn: String): Option[File] = {
          val oldNameInTmpDir: File = new File(System.getProperty("java.io.tmpdir"), originalNameIn)
          if (oldNameInTmpDir.getParentFile.canWrite && !oldNameInTmpDir.exists()) Some(oldNameInTmpDir)
          else {
            val (baseName, extension) = controller.getReplacementFilename(originalPathIn)
            Some(File.createTempFile(baseName + "-", extension))
          }
        }
        val originalName = FilenameUtils.getBaseName(originalPathIn)
        if (!origPathFile.getParentFile.exists) {
          newLocation(originalName)
        } else {
          val msgIfExists = if (!new File(originalPathIn).exists) "" else " (overwriting the current copy)"
          val ans = askYesNoQuestion("Put the file in the original location: \"" + originalPathIn + "\"" + msgIfExists + "?")
          if (ans.isEmpty) None
          else if (ans.get) Some(origPathFile)
          else newLocation(originalName)
        }
      }
    }
  }

}
