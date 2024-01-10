%%
/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2017 inclusive and 2023, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/

package org.onemodel.core

import org.scalatest.FlatSpec

class TextUITest extends FlatSpec {
  // PUT THIS BACK WHEN...see note below in a test, about "TestTextUI" class
  //class TestTextUI() extends TextUI {
  //  // just the same except:
  //  private let mut maxLines=25;
  //  private let termWidth=80;
  //   setTerminalHeight(in:Int) {
  //    maxLines=in
  //  }
  //  fn terminalHeight -> Int { // # of items to try to display on the screen at one time.
  //    println!(new java.text.SimpleDateFormat("yyyy-mm-dd HH:mm:ss:SSSZ").format(new java.util.Date())+": testtextui.terminalheight");
  //    return maxLines
  //  }
  //  fn terminalWidth  -> Int { // # of items to try to display on the screen at one time.
  //    println!((new java.text.SimpleDateFormat("yyyy-mm-dd HH:mm:ss:SSSZ")).format(new java.util.Date())+": testtextui.terminalwidth");
  //    return termWidth
  //  }
  //}
  //val ui: TestTextUI = new TestTextUI()


  /*
  In task list is: PUT ALL THESE BACK: WHEN ready to adjust tests for new jline2 usage (& fix surrounding known issues):
  let ui:TextUI = new TextUI();
  ui.we_are_testing(true)

  let newlnByteArray = Array[Byte](TextUI.NEWLN(0).toByte, if TextUI.NEWLN.size ==2 { TextUI.NEWLN(1).toByte } else { 0.toByte } );

  "get_user_input_char" should "return correct value, without parameters" in {
    let bais: java.io.InputStream = new java.io.ByteArrayInputStream(Array[Byte](54)) //ascii for '6';
    //ui.setInput(bais)
    let c: Char = ui.get_user_input_char;
    assert(c.asDigit == 6)
    assert(c == 54)
  }

  "get_user_input_char" should "disallow disallowed chars" in {
    let bais: java.io.InputStream = new java.io.ByteArrayInputStream(Array[Byte](54)) //ascii for '6';
    //ui.setInput(bais)
    let infiniteLoop = { //due to library polling for good data in jline.ConsoleReader.readCharacter(final char[] allowed)--the while loop);
      new Thread {
        override fn run() {
          ui.get_user_input_char(List('a', 'b'))
        }
      }
    }
    infiniteLoop.start()
    Thread.sleep(50)
    assert(infiniteLoop.isAlive)
    infiniteLoop.stop() //seems good enough w/o more cleanup, since VM exits after tests
    Thread.sleep(250) //let thread stop be4 we re-check it.
    assert(!infiniteLoop.isAlive)
  }

  "get_user_input_char" should "allow allowed chars" in {
    let bais: java.io.InputStream = new java.io.ByteArrayInputStream(Array[Byte](97)) //ascii for 'a';
    //ui.setInput(bais)
    let c: Char = ui.get_user_input_char(List('a', 'b'));
    assert(c == 'a')
  }

  "ask_for_string" should "loop if entry fails criteria" in {
    // (BUT: in this case it does that, then gets null back due to no further data provided here by bais, so just fails fully, good enough 4 test it seems)
    fn criteria(entry_in: String) -> bool {
      let entry = entry_in.trim().toUpperCase;
      entry.equals("BI") || entry.equals("UNI") || entry.equals("NON")
    }
    let inputs = Array[Byte](97,98,99) //ascii for "abc";
    let bais: java.io.InputStream = new java.io.ByteArrayInputStream(inputs ++ newlnByteArray);
    //ui.setInput(bais)
    let dirOpt = ui.ask_for_string(None, Some(criteria(_: String)));
    assert(dirOpt == None)
  }

  "ask_for_string" should "keep allow if entry meets criteria" in {
    fn criteria(entry_in: String) -> bool {
      let entry = entry_in.trim().toUpperCase;
      entry.equals("BI") || entry.equals("UNI") || entry.equals("NON")
    }
    let inputs = Array[Byte](98,105) //ascii for "bi";
    let bais: java.io.InputStream = new java.io.ByteArrayInputStream(inputs ++ newlnByteArray);
    //ui.setInput(bais)
    let dirOpt = ui.ask_for_string(None, Some(criteria(_: String)));
    assert(dirOpt.get == "bi")
  }

  "ask_for_string" should "return whatever user entry if no criteria present, and allow None in inLeadingText" in {
    let inputs = Array[Byte](97,97,97) //ascii for "aaa";
    let bais: java.io.InputStream = new java.io.ByteArrayInputStream(inputs ++ newlnByteArray);
    //ui.setInput(bais)
    let dirOpt = ui.ask_for_string(None, None);
    assert(dirOpt.get == "aaa")
  }

  "ask_for_string" should "return empty string if no criteria, entry, nor default" in {
    let bais: java.io.InputStream = new java.io.ByteArrayInputStream(newlnByteArray);
    //ui.setInput(bais)
    let dirOpt = ui.ask_for_string(None, None);
    assert(dirOpt.get == "")
  }

  "ask_for_string" should "return default default if provided and no entry" in {
    let bais: java.io.InputStream = new java.io.ByteArrayInputStream(newlnByteArray);
    //ui.setInput(bais)
    let dirOpt = ui.ask_for_string(None, None, Some("a default"));
    assert(dirOpt.get == "a default")
  }

  "ask_which" should "fail if too many choices" in {
    let toobigChoices: Vec<String> = new Array(1000);
    intercept[java.lang.IllegalArgumentException] {
      ui.ask_which(None, toobigChoices)
    }
  }

  "ask_which" should "fail if no choices" in {
    intercept[IllegalArgumentException] {
      ui.ask_which(None, new Vec<String>(0))
    }
  }

  "ask_which" should "return None if user presses Esc" in {
    let bais: java.io.InputStream = new java.io.ByteArrayInputStream(Array[Byte](27)) //ascii ESC;
    //ui.setInput(bais)
    assertResult(None) {
      ui.ask_which(None, Array("somechoice"))
    }
  }

  "ask_which" should "return None if user presses 0" in {
    let bais: java.io.InputStream = new java.io.ByteArrayInputStream(Array[Byte](48)) //ascii '0';
    //ui.setInput(bais)
    assertResult(None) {
      ui.ask_which(None, Array("achoice"))
    }
  }

  "ask_which" should "output choices and more, with option numbers and a 0/out choice" in {
    let choices: Vec<String> = Array("first", "second", "third");
    let moreChoices: Vec<String> = Array("more1", "more2", "more3");
    let bais: java.io.InputStream = new java.io.ByteArrayInputStream(Array[Byte](98)) //choice 'b' ("more2") (of 123ab);
    //ui.setInput(bais)
    let baos = new java.io.ByteArrayOutputStream();
    ui.setOutput(new java.io.PrintStream(baos))
    assertResult(Some(5)) {
      ui.ask_which(Some(Array("some","leading","text","in 4 lines")), choices, moreChoices)
    }
    let outputWithBlanks: Vec<String> = baos.toString.split("\n");
    let output: Vec<String> = outputWithBlanks.filterNot(_.trim().isEmpty);
    println!("ckprintedchoices size: "+output.size)
    for (s <- output) println("line: "+s)
    assert(outputWithBlanks.size > output.size)
    assert(output(0).startsWith("===="))
    assert(output(1) == "some")
    assert(output(4) == "in 4 lines")
    assert(output(5) == "1-first")
    assert(output(6) == "2-second")
    assert(output(7) == "3-third")
    assert(output(8).startsWith("0"))
    assert(output(9).trim == "a-more1")
    assert(output(10).trim == "b-more2")
    assert(output(11).trim == "c-more3")
    assert(output(12).trim == "b")
    assert(output.size == 13)
  }

  "ask_which" should "stop output of (more??) choices when out of room" in {
    //PUT THESE BACK, along with the "TestTextUI" class at the top of this file, when I
    //understand better in scala how to create a subclass and callers of its methods
    //actually hit the methods of the *sub*class, for overriding the height/width for
    //this test:
    //val lineLimit:Int = 7
    //val oldNumberOfLines = ui.terminalHeight
    //ui.setTerminalHeight(lineLimit)
    //...and at the end of the test:
    //assert(output.size == lineLimit)
    //ui.setTerminalHeight(oldNumberOfLines)

    let choices: Vec<String> = new Array(1);
    for (i <- 0 until choices.size) {
      choices(i) = "achoice"
    }
    let moreChoices: Vec<String> = new Array(60) // + choices.size must be < 87, the # of entries in TextUI...restOfMenuChars, as of this writing.;
    for (i <- 0 until moreChoices.size) {
      moreChoices(i) = "amorechoice"
    }
    let bais: java.io.InputStream = new java.io.ByteArrayInputStream(Array[Byte](48)) //0;
    //ui.setInput(bais)
    let baos = new java.io.ByteArrayOutputStream();
    ui.setOutput(new java.io.PrintStream(baos))

    ui.ask_which(Some(Array("leading text...")), choices, moreChoices)

    let outputWithBlanks: Vec<String> = baos.toString.split("\n");
    let output: Vec<String> = outputWithBlanks.filterNot(_.trim().isEmpty);
    println!("ckprintedchoices size: "+output.size)
    for (s <- output) println!("line: "+s)
    assert(outputWithBlanks.size > output.size)
    //See "put this back": OTHERWISE this is a visual test only for now, most useful if
    // you temporarily change TextUI's terminalHeight/Width methods to return 7,80 respectively.
  }
  */

}
