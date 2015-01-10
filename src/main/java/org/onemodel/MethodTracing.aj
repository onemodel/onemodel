/*  This file is part of OneModel, a program to manage knowledge.  
    Copyright in each year of 2013-2015 inclusive, Luke A Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel;

import org.aspectj.lang.JoinPoint;
import org.aspectj.lang.reflect.CodeSignature;

/** This is an attempt to make debugging work along the lines of "set -x"
    in bash code.  This can be modified to get only certain log output.
    There's probably a way to do it at runtime w/o recompilation, but this is
    what is working now.

    **************************
    When changing this class, one has to run 'mvn clean' once, then 'mvn package' 
    *TWICE* (such as in the script bin/c), for reasons left to the future.
    FOR MORE INFO ABOUT ASPECTS IN THE CURRENT IMPLEMENTATION, SEE comments
    in the pom.xml. 
    **************************
    
    TO USE:  at the "before()" & "after() methods, comment out BOTH lines 
    that say "/*"  but don't commit that change.  I.e., change "/*" to "// /*" 
    or "///*".
*/
public aspect MethodTracing {
  int depth = 0;
  StringBuffer callDepthSpacing = new StringBuffer("");
  final static String NEWLN = System.getProperty("line.separator");
  
  pointcut allOMMethods():
    execution(* org.onemodel..*(..))
    && !within(MethodTracing)
    ;
  
  before(): allOMMethods() {
    /*
     * // indented & marked for easy visual separation of this from other output
     * callDepthSpacing.append("  "); depth++;
     * System.out.println(callDepthSpacing + ">entering(" + depth + ") " +
     * thisJoinPoint + ", " + thisJoinPoint.getSourceLocation());
     * 
     * // also occasionally useful to debug: printParameters(thisJoinPoint);
     */
  }
  
  after() returning: allOMMethods() {
    /*
     * // the spacing should line up w/ "entering" above
     * System.out.println(callDepthSpacing + "<exiting (" + depth + ") " +
     * thisJoinPoint + ", " + thisJoinPoint.getSourceLocation());
     * callDepthSpacing.delete(0, 2); depth--; //
     */
  }
  
  // initially from: file:///usr/share/doc/aspectj-doc/progguide/examples-basic.html
  private void printParameters(JoinPoint jp) {
    System.out.println(callDepthSpacing + "  Arguments: " );
    Object[] args = jp.getArgs();
    String[] names = ((CodeSignature)jp.getSignature()).getParameterNames();
    @SuppressWarnings("rawtypes")
    Class[] types = ((CodeSignature)jp.getSignature()).getParameterTypes();
    for (int i = 0; i < args.length; i++) {
      String display ="";
      if (args[i] != null) {
        if (args[i] instanceof String[]) {
          String[] strs = (String[]) args[i];
          for (String s : strs) {
            if (display.length() == 0) display += "  \"" + s + "\"";
            else display += NEWLN + "  \"" + s + "\"";
          }
        }
        else if (args[i].getClass().isArray()) {
          for (Object arg : args) {
            if (display.length() == 0) display += "  " + arg;
            else display += NEWLN + "  " + arg;
          }
        }
        else if (args[i] instanceof String) display = "\"" + args[i].toString()
            + "\"";
        else if (args[i] instanceof scala.Option) {
          if (args[i].toString() == "None") display = args[i].toString();
          else display = (((scala.Option) (args[i])).get().toString());
        }
        else {
          display = args[i].toString();
        }
      }
      System.out.println(callDepthSpacing + "    "  + i + ". " + names[i] +
          " : " + types[i].getName() +
          " = "
          + display
          + (args[i] == null ? "" : 
            (": \"" + 
              ((args[i] instanceof String) ? ((String) args[i]).toString() + "/" + args[i] :
                args[i].toString()) +
            "\"")));
    }
  }
}
