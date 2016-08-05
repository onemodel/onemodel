/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, and 2013-2016 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  If we ever do port to another database, create the Database interface (removed around 2014-1-1 give or take) and see other changes at that time.
  An alternative method is to use jdbc escapes (but this actually might be even more work?):  http://jdbc.postgresql.org/documentation/head/escapes.html  .
  Another alternative is a layer like JPA, ibatis, hibernate  etc etc.

*/
package org.onemodel.database

import java.io.{PrintWriter, StringWriter}
import java.sql.{Connection, DriverManager, ResultSet, Statement}

import org.onemodel.controller.Controller
import org.onemodel.model._
import org.onemodel.{OmDatabaseException, OmFileTransferException}
import org.postgresql.largeobject.{LargeObject, LargeObjectManager}

import scala.annotation.tailrec
import scala.collection.mutable
import scala.util.Sorting

/** Some methods are here on the object, so that PostgreSQLDatabaseTest can call destroyTables on test data.
  */
object PostgreSQLDatabase {
  // should these be more consistently upper-case? What is the scala style for constants?  similarly in other classes.
  val CURRENT_DB_VERSION = 2
  val dbNamePrefix = "om_"
  val MIXED_CLASSES_EXCEPTION = "All the entities in a group should be of the same class."
  // so named to make it unlikely to collide by name with anything else:
  val systemEntityName = ".system-use-only"
  // aka template entities:
  val classTemplateEntityGroupName = "class-defining entities"
  val theHASrelationTypeName = "has"
  val theIsHadByReverseName = "is had by"
  val EDITOR_INFO_ENTITY_NAME = "editorInfo"
  val TEXT_EDITOR_INFO_ENTITY_NAME = "textEditorInfo"
  val TEXT_EDITOR_COMMAND_ATTRIBUTE_TYPE_NAME = "textEditorCommand"
  val PREF_TYPE_BOOLEAN = "boolean"
  val PREF_TYPE_ENTITY_ID = "entityId"

  // where we create the table also calls this.
  // Longer than the old 60 (needed), and a likely familiar length to many people (for ease in knowing when done), seems a decent balance. If any longer
  // is needed, maybe it should be put in a TextAttribute and make those more convenient to use, instead.
  def entityNameLength: Int = 160

  // in postgres, one table "extends" the other (see comments in createTables)
  def relationTypeNameLength: Int = entityNameLength

  def classNameLength: Int = entityNameLength


  def destroyTables(inDbNameWithoutPrefix: String, username: String, password: String) {
    Class.forName("org.postgresql.Driver")
    val conn: Connection = DriverManager.getConnection("jdbc:postgresql:" + dbNamePrefix + inDbNameWithoutPrefix, username, password)
    conn.setTransactionIsolation(Connection.TRANSACTION_SERIALIZABLE)
    destroyTables_helper(conn)
  }

  private def destroyTables_helper(connIn: Connection) {
    // Doing these individually so that if one fails (not previously existing, such as testing or a new installation), the others can proceed (drop method
    // ignores that exception).

    drop("table", "om_db_version", connIn)
    drop("table", Controller.QUANTITY_TYPE, connIn)
    drop("table", Controller.DATE_TYPE, connIn)
    drop("table", Controller.BOOLEAN_TYPE, connIn)
    // The next line is to invoke the trigger that will clean out Large Objects (FileAttributeContent...) from the table pg_largeobject.
    // The LO cleanup doesn't happen (trigger not invoked) w/ just a drop (or truncate), but does on delete.  For more info see the wiki reference
    // link among those down in this file below "create table FileAttribute".
    try {
      dbAction("delete from FileAttributeContent", callerChecksRowCountEtc = true, connIn = connIn)
    } catch {
      case e: Exception =>
        val sw: StringWriter = new StringWriter()
        e.printStackTrace(new PrintWriter(sw))
        val messages = sw.toString
        if (!messages.contains("does not exist")) throw e
    }
    drop("table", "FileAttributeContent", connIn)
    drop("table", Controller.FILE_TYPE, connIn)
    drop("table", Controller.TEXT_TYPE, connIn)
    drop("table", Controller.RELATION_TO_ENTITY_TYPE, connIn)
    drop("table", "EntitiesInAGroup", connIn)
    drop("table", Controller.RELATION_TO_GROUP_TYPE, connIn)
    drop("table", "action", connIn)
    drop("table", "grupo", connIn)
    drop("table", Controller.RELATION_TYPE_TYPE, connIn)
    drop("table", "AttributeSorting", connIn)
    drop("table", Controller.ENTITY_TYPE, connIn)
    drop("table", "class", connIn)
    drop("sequence", "EntityKeySequence", connIn)
    drop("sequence", "ClassKeySequence", connIn)
    drop("sequence", "TextAttributeKeySequence", connIn)
    drop("sequence", "QuantityAttributeKeySequence", connIn)
    drop("sequence", "RelationTypeKeySequence", connIn)
    drop("sequence", "ActionKeySequence", connIn)
    drop("sequence", "RelationToEntityKeySequence", connIn)
    drop("sequence", "RelationToGroupKeySequence", connIn)
    drop("sequence", "RelationToGroupKeySequence2", connIn)
    drop("sequence", "DateAttributeKeySequence", connIn)
    drop("sequence", "BooleanAttributeKeySequence", connIn)
    drop("sequence", "FileAttributeKeySequence", connIn)
  }

  private def drop(sqlType: String, name: String, connIn: Connection) {
    try dbAction("drop " + escapeQuotesEtc(sqlType) + " " + escapeQuotesEtc(name) + " CASCADE", callerChecksRowCountEtc = false, connIn = connIn)
    catch {
      case e: Exception =>
        val sw: StringWriter = new StringWriter()
        e.printStackTrace(new PrintWriter(sw))
        val messages = sw.toString
        if (!messages.contains("does not exist")) throw e
    }
  }

  def getAttributeFormId(key: String): Int = {
    //MAKE SURE THESE MATCH WITH THOSE IN attributeKeyExists and getAttributeFormName !
    key match {
      case Controller.QUANTITY_TYPE => 1
      case Controller.DATE_TYPE => 2
      case Controller.BOOLEAN_TYPE => 3
      case Controller.FILE_TYPE => 4
      case Controller.TEXT_TYPE => 5
      case Controller.RELATION_TO_ENTITY_TYPE => 6
      case Controller.RELATION_TO_GROUP_TYPE => 7
    }
  }
  def getAttributeFormName(key: Int): String = {
    // MAKE SURE THESE MATCH WITH THOSE IN getAttributeFormId !
    //idea: put these values in a structure that is looked up both ways, instead of duplicating them?
    key match {
      case 1 => Controller.QUANTITY_TYPE
      case 2 => Controller.DATE_TYPE
      case 3 => Controller.BOOLEAN_TYPE
      case 4 => Controller.FILE_TYPE
      case 5 => Controller.TEXT_TYPE
      case 6 => Controller.RELATION_TO_ENTITY_TYPE
      case 7 => Controller.RELATION_TO_GROUP_TYPE
    }
  }

  /** For text fields (which by the way should be surrounded with single-quotes ').  Best to use this
    * with only one field at a time, so you don't escape the single-ticks that *surround* the field.
    */
  def escapeQuotesEtc(s: String): String = {
    var result: String = s
    /*
    //both of these seem to work to embed a ' (single quote) in interactive testing w/ psql: the SQL standard
    //way (according to http://www.postgresql.org/docs/9.1/interactive/sql-syntax-lexical.html#SQL-SYNTAX-STRINGS )
    //    update entity set (name) = ('len''gth4') where id=-9223372036854775807;
    //...or the postgresql extension way (also works for: any char (\a is a), c-like (\b, \f, \n, \r, \t), or
    //hex (eg \x27), or "\u0027 (?) , \U0027 (?)  (x = 0 - 9, A - F)  16 or 32-bit
    //hexadecimal Unicode character value"; see same url above):
    //    update entity set (name) = (E'len\'gth4') where id=-9223372036854775807;
    */
    // we don't have to do much: see the odd string that works ok, searching for "!@#$%" etc in PostgreSQLDatabaseTest.
    result = result.replaceAll("'", "\39")
    // there is probably a different/better/right way to do this, possibly using the psql functions quote_literal or quote_null,
    // or maybe using "escape" string constants (a postgresql extension to the sql standard). But it needs some thought, and maybe
    // this will work for now, unless someone needs to access the DB in another form. Kludgy, yes. It's on the fix list.
    result = result.replaceAll(";", "\59")
    result
  }

  def unEscapeQuotesEtc(s: String): String = {
    // don't have to do the single-ticks ("'") because the db does that automatically when returning data (see PostgreSQLDatabaseTest).

    var result: String = s
    result = result.replaceAll("\39", "'")
    result = result.replaceAll("\59", ";")
    result
  }

  /** Returns the # of rows affected.
    * @param skipCheckForBadSqlIn  SET TO false EXCEPT *RARELY*, WITH CAUTION AND ONLY WHEN THE SQL HAS NO USER-PROVIDED STRING IN IT!!  SEE THE (hopefully
    *                              still just one) PLACE USING IT NOW (in method createAttributeSortingDeletionTrigger) AND PROBABLY LIMIT USE TO THAT!
    */
  def dbAction(sqlIn: String, callerChecksRowCountEtc: Boolean = false, connIn: Connection, skipCheckForBadSqlIn: Boolean = false): Long = {
    var rowsAffected = -1
    var st: Statement = null
    val isCreateDropOrAlterStatement = sqlIn.toLowerCase.startsWith("create ") || sqlIn.toLowerCase.startsWith("drop ") ||
                                       sqlIn.toLowerCase.startsWith("alter ")
    try {
      st = connIn.createStatement
      if (! skipCheckForBadSqlIn) {
        checkForBadSql(sqlIn)
      }
      rowsAffected = st.executeUpdate(sqlIn)

      // idea: not sure whether these checks belong here really.  Might be worth research
      // to see how often warnings actually should be addressed, & how to routinely tell the difference. If so, do the same at the
      // other place(s) that use getWarnings.
      val warnings = st.getWarnings
      if (warnings != null
          && !warnings.toString.contains("NOTICE: CREATE TABLE / PRIMARY KEY will create implicit index")
          && !warnings.toString.contains("NOTICE: drop cascades to constraint valid_related_to_entity_id on table class")
      ) {
        throw new OmDatabaseException("Warnings from postgresql. Matters? Says: " + warnings)
      }
      if (!callerChecksRowCountEtc && !isCreateDropOrAlterStatement && rowsAffected != 1) {
        throw new OmDatabaseException("Affected " + rowsAffected + " rows instead of 1?? SQL was: " + sqlIn)
      }
      rowsAffected
    } catch {
      case e: Exception =>
        val msg = "Exception while processing sql: "
        throw new OmDatabaseException(msg + sqlIn, e)
    } finally {
      if (st != null) st.close()
    }
  }

  def checkForBadSql(s: String) {
    if (s.contains(";")) {
      // it seems that could mean somehow an embedded sql is in a normal command, as an attack vector. We don't usually need
      // to write like that, nor accept it from outside. This & any similar needed checks should happen reliably
      // at the lowest level before the database for security.  If text needs the problematic character(s), it should
      // be escaped prior (see escapeQuotesEtc for writing data, and where we read data).
      throw new OmDatabaseException("Input can't contain ';'")
    }
  }

}


/**
 * Any code that would change when we change storage systems (like from postgresql to
 * an object database or who knows), goes in this class.
 * <br><br>
 * Note that any changes to the database structures (or constraints, etc) whatsoever should
 * ALWAYS have the following: <ul>
 * <li>Constraints, rules, functions, stored procedures, or triggers
 * or something to enforce data integrity and referential integrity at the database level,
 * whenever possible. When this is impossible, it should be discussed on the developer mailing
 * so that we can consider putting it in the right place in the code, with the goal of
 * greatest simplicity and reliability.</li>
 * <li>Put these things in the auto-creation steps of the DB class. See createBaseData(), createTables(), and doDatabaseUpgrades.</li>
 * <li>Add comments to that part of the code, explaining the change or requirement, as needed.</li>
 * <li>Any changes (as anywhere in this system) should be done in a test-first manner, for anything that
 * could go wrong, along these lines: First write a test that demonstrates the issue and fails, then
 * write code to correct the issue, then re-run the test to see the successful outcome. This helps keep our
 * regression suite current, and could even help think through design issues without over-complicating things.
 * </ul>
 *
 * This creates a new instance of Database. By default, auto-commit is on unless you explicitly open a transaction; then
 * auto-commit will be off until you rollbackTrans() or commitTrans(), at which point auto-commit is
 * turned back on.
 */
class PostgreSQLDatabase(username: String, var password: String) {
  private val ENTITY_ONLY_SELECT_PART: String = "SELECT e.id"
  protected var mConn: Connection = null
  // When true, this means to override the usual settings and show the archived entities too (like a global temporary "un-archive"):
  private var mShowAllArchivedEntities = false

  Class.forName("org.postgresql.Driver")
  connect(PostgreSQLDatabase.dbNamePrefix + username, username, password)
  // clear the password from memory. Is there a better way?:
  password = null
  System.gc()
  System.gc()
  if (!modelTablesExist) {
    createTables()
    createBaseData()
  }
  doDatabaseUpgradesIfNeeded()
  createExpectedData()

  /** Creates newly-assumed data in existing systems.  I.e., not a database schema change, and was added to the system (probably expected by the code somewhere),
    * after an OM release was done.  This puts it into existing databases if needed.
    */
  def createExpectedData(): Unit = {
    //Idea: should this really be in the controller then?  It wouldn't differ by which database type we are using.  Hmm, no, if there were multiple
    // database types, there would probably a parent class over them (of some kind) to hold this.
    val systemEntityId = getSystemEntityId
    val HASrelationTypeId = findRelationType(PostgreSQLDatabase.theHASrelationTypeName, Some(1))(0)

    val preferencesContainerId: Long = {
      val preferencesEntityId: Option[Long] = getRelationToEntityByName(getSystemEntityId, Controller.USER_PREFERENCES)
      if (preferencesEntityId.isDefined) {
        preferencesEntityId.get
      } else {
        // Since necessary, also create the entity that contains all the preferences:
        val newEntityId: Long = createEntityAndRelationToEntity(systemEntityId, HASrelationTypeId, Controller.USER_PREFERENCES, None,
                                                                Some(System.currentTimeMillis), System.currentTimeMillis)._1
        newEntityId
      }
    }
    // (Not doing the default entity preference here also, because it might not be set by not and is not assumed to be.)
    if (getUserPreference2(preferencesContainerId, Controller.SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE, PostgreSQLDatabase.PREF_TYPE_BOOLEAN).isEmpty) {
      setUserPreference_Boolean(Controller.SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE, valueIn = false)
    }
  }

  def connect(inDbNameWithoutPrefix: String, username: String, password: String) {
    try if (mConn != null) mConn.close()
    catch {case e: Exception => throw new RuntimeException(e)}
    mConn = DriverManager.getConnection("jdbc:postgresql:" + inDbNameWithoutPrefix, username, password)
    mConn.setTransactionIsolation(Connection.TRANSACTION_SERIALIZABLE)
  }

  /** @param skipCheckForBadSqlIn   Avoid using this parameter! See comment on PostgreSQLDatabase.dbAction.
    */
  def dbAction(sqlIn: String, callerChecksRowCountEtc: Boolean = false, skipCheckForBadSqlIn: Boolean = false): Long = {
    PostgreSQLDatabase.dbAction(sqlIn, callerChecksRowCountEtc, mConn, skipCheckForBadSqlIn)
  }

  /** Does standard setup for a "OneModel" database, such as when starting up for the first time, or when creating a test system. */
  def createTables() {
    beginTrans()
    try {
      createVersionTable()

      dbAction("create sequence EntityKeySequence minvalue " + minIdValue)

      // id must be "unique not null" in ANY database used, because it is a primary key. "PRIMARY KEY" is the same.
      dbAction("create table Entity (" +
               "id bigint DEFAULT nextval('EntityKeySequence') PRIMARY KEY, " +
               "name varchar(" + PostgreSQLDatabase.entityNameLength + ") NOT NULL, " +
               "class_id bigint, " +
               // 'archived' is only on Entity for now, to see if rows from related tables just don't show up because we
               // never link to them (never seeing the linking Entity rows), so they're effectively hidden/archived too.
               // At some point we could consider moving all those rows (entities & related...) to separate tables instead,
               // for performance/space if needed (including 'public').
               "archived boolean NOT NULL default false, " +
               "archived_date bigint check ((archived is false and archived_date is null) OR (archived and archived_date is not null)), " +
               // intended to be a readonly date: the (*java*-style numeric: milliseconds since 1970-1-1 or such) when this row was inserted (ie, when the
               // entity object was created in the db):
               "insertion_date bigint not null, " +
               // null in the 'public' field means 'undecided' (effectively "false", but a different nuance,e.g. in case user wants to remember to decide later)
               "public boolean " +
               ") ")
      // not unique, but for convenience/speed:
      dbAction("create index entity_lower_name on Entity (lower(NAME))")

      dbAction("create sequence ClassKeySequence minvalue " + minIdValue)

      // the name here doesn't have to be the same name as in the related Entity record, (since it's not a key, and it might not make sense to match).
      // For additional comments on usage, see the Controller.askForInfoAndCreateEntity method.
      dbAction("create table Class (" +
               "id bigint DEFAULT nextval('ClassKeySequence') PRIMARY KEY, " +
               "name varchar(" + PostgreSQLDatabase.classNameLength + ") NOT NULL, " +
               // In other words, template, aka class-defining entity:
               "defining_entity_id bigint UNIQUE NOT NULL, " +
               // this means whether the user wants the program to create all the attributes by default, using the defining_entity's attrs as a template:
               "create_default_attributes boolean, " +
               "CONSTRAINT valid_related_to_entity_id FOREIGN KEY (defining_entity_id) REFERENCES entity (id) " +
               ") ")
      dbAction("alter table entity add CONSTRAINT valid_related_to_class_id FOREIGN KEY (class_id) REFERENCES class (id)")


      dbAction("create sequence RelationTypeKeySequence minvalue " + minIdValue)
      // this table "inherits" from Entity (each relation type is an Entity) but we use homegrown "inheritance" for that to make it
      // easier to port to databases that don't have postgresql-like inheritance built in. It inherits from Entity so that as Entity
      // expands (i.e., context-based naming or whatever), we'll automatically get the benefits, in objects based on this table (at least
      // that's the idea at this moment...) --Luke Call 8/2003  That may have been a mistake, more of a nuisance to coordinate
      // them than having 2 tables (luke, 2013-11-1).
      // inherits from Entity; see RelationConnection for more info.
      // Note, 2014-07: At one point I considered whether this concept overlaps with that of class, but now I think they are quite separate.  This table
      // could fill the concept of an entity that *is* a relationship, containing e.g. the date a relationship began, or any other attributes that are not about
      // either participant, but about the relationship itself.  One such use could be: I "have" a physical object, I and the object being entities with
      // classes, and the "have" is not a regular generic "have" type (as defined by the system at first startup), but a particular one (maybe RelationType
      // should be renamed to "RelationEntity" or something: think about all this some more: more use cases etc).
      dbAction("create table RelationType (" +
               "entity_id bigint PRIMARY KEY, " +
               "name_in_reverse_direction varchar(" + PostgreSQLDatabase.relationTypeNameLength + "), " +
               // valid values are "BI ","UNI","NON"-directional for this relationship. example: parent/child is unidirectional. sibling is bidirectional,
               // and for nondirectional
               // see Controller's mention of "nondir" and/or elsewhere for comments
               "directionality char(3) CHECK (directionality in ('BI','UNI','NON')), " +
               "CONSTRAINT valid_rel_entity_id FOREIGN KEY (entity_id) REFERENCES Entity (id) ON DELETE CASCADE " +
               ") ")


      /* This table maintains the users' preferred display sorting information for entities' attributes (including relations to groups/entities).

         It might instead have been implemented by putting the sorting_index column on each attribute table, which would simplify some things, but that
         would have required writing a new way for placing & sorting the attributes and finding adjacent ones etc., and the first way was already
         mostly debugged, with much effort (for EntitiesInAGroup, and the hope is to reuse that way for interacting with this table).  But maybe that
         same effect could have been created by sorting the attributes in memory instead, adhoc when needed: not sure if that would be simpler
      */
      dbAction("create table AttributeSorting (" +
               // the entity whose attribute this is:
               "entity_id bigint NOT NULL" +
               // next field is for which table the attribute is in.  Method getAttributeForm has details.
               ", attribute_form_id smallint NOT NULL" +
               ", attribute_id bigint NOT NULL" +
               // the reason for this table:
               ", sorting_index bigint not null" +
               ", PRIMARY KEY (entity_id, attribute_form_id, attribute_id)" +
               ", CONSTRAINT valid_entity_id FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE" +
               ", CONSTRAINT valid_attribute_form_id CHECK (attribute_form_id >= 1 AND attribute_form_id <= 7)" +
               // make it so the sorting_index must also be unique for each entity (otherwise we have sorting problems):
               ", constraint noDupSortingIndexes2 unique (entity_id, sorting_index)" +
               // this one was required by the constraint valid_*_sorting on the tables that have a form_id column:
               ", constraint noDupSortingIndexes3 unique (attribute_form_id, attribute_id)" +
               ") ")
      dbAction("create index AttributeSorting_sorted on AttributeSorting (entity_id, sorting_index)")
      createAttributeSortingDeletionTrigger()

      dbAction("create sequence QuantityAttributeKeySequence minvalue " + minIdValue)
      // The entity_id is the key for the entity on which this quantity info is recorded; for other meanings see comments on
      // Entity.addQuantityAttribute(...).
      // id must be "unique not null" in ANY database used, because it is the primary key.
      // FOR COLUMN MEANINGS, SEE ALSO THE COMMENTS IN CREATEQUANTITYATTRIBUTE.
      dbAction("create table QuantityAttribute (" +
               // see comment for this column under "create table RelationToEntity", below:
               "form_id smallint DEFAULT " + PostgreSQLDatabase.getAttributeFormId(Controller.QUANTITY_TYPE) +
               "    NOT NULL CHECK (form_id=" + PostgreSQLDatabase.getAttributeFormId(Controller.QUANTITY_TYPE) + "), " +
               "id bigint DEFAULT nextval('QuantityAttributeKeySequence') PRIMARY KEY, " +
               "entity_id bigint NOT NULL, " +
               //refers to a unit (an entity), like "meters":
               "unit_id bigint NOT NULL, " +
               // eg, 50.0:
               "quantity_number double precision not null, " +
               //eg, length (an entity):
               "attr_type_id bigint not null, " +
               // see "create table RelationToEntity" for comments about dates' meanings.
               "valid_on_date bigint, " +
               "observation_date bigint not null, " +
               "CONSTRAINT valid_unit_id FOREIGN KEY (unit_id) REFERENCES entity (id), " +
               "CONSTRAINT valid_attr_type_id FOREIGN KEY (attr_type_id) REFERENCES entity (id), " +
               "CONSTRAINT valid_parent_id FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE, " +
               // Didn't use "on delete cascade" for the following constraint, because it didn't originally occur to me that instead of deleting the
               // sorting row (via triggers) when we delete the attribute, we could delete the attribute when deleting its sorting row, by instead
               // putting "ON DELETE CASCADE" on the attribute tables' constraints that reference this table, and where we
               // now delete attributes, instead deleting AttributeSorting rows, and so letting the attributes be deleted automatically.
               // But for now, see the trigger below instead.
               // (The same is true for all the attribute tables (including the 2 main RelationTo* tables).
               "CONSTRAINT valid_qa_sorting FOREIGN KEY (entity_id, form_id, id) REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) " +
               // (next line is because otherwise when an attribute is deleted, it would fail on this constraint before the trigger files to delete the
               // row from attributesorting.)
               "  DEFERRABLE INITIALLY DEFERRED " +
               ") ")
      dbAction("create index quantity_parent_id on QuantityAttribute (entity_id)")
      dbAction("CREATE TRIGGER qa_attribute_sorting_cleanup BEFORE DELETE ON QuantityAttribute " +
               "FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()")

      dbAction("create sequence DateAttributeKeySequence minvalue " + minIdValue)
      dbAction("create table DateAttribute (" +
               // see comment for this column under "create table RelationToEntity", below:
               "form_id smallint DEFAULT " + PostgreSQLDatabase.getAttributeFormId(Controller.DATE_TYPE) +
               "    NOT NULL CHECK (form_id=" + PostgreSQLDatabase.getAttributeFormId(Controller.DATE_TYPE) + "), " +
               "id bigint DEFAULT nextval('DateAttributeKeySequence') PRIMARY KEY, " +
               "entity_id bigint NOT NULL, " +
               //eg, due on, done on, should start on, started on on... (which would be an entity)
               "attr_type_id bigint not null, " +
               "date bigint not null, " +
               "CONSTRAINT valid_attr_type_id FOREIGN KEY (attr_type_id) REFERENCES entity (id), " +
               "CONSTRAINT valid_parent_id FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE, " +
               "CONSTRAINT valid_da_sorting FOREIGN KEY (entity_id, form_id, id) REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) " +
               "  DEFERRABLE INITIALLY DEFERRED " +
               ") ")
      dbAction("create index date_parent_id on DateAttribute (entity_id)")
      dbAction("CREATE TRIGGER da_attribute_sorting_cleanup BEFORE DELETE ON DateAttribute " +
               "FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()")

      dbAction("create sequence BooleanAttributeKeySequence minvalue " + minIdValue)
      dbAction("create table BooleanAttribute (" +
               // see comment for this column under "create table RelationToEntity", below:
               "form_id smallint DEFAULT " + PostgreSQLDatabase.getAttributeFormId(Controller.BOOLEAN_TYPE) +
               "    NOT NULL CHECK (form_id=" + PostgreSQLDatabase.getAttributeFormId(Controller.BOOLEAN_TYPE) + "), " +
               "id bigint DEFAULT nextval('BooleanAttributeKeySequence') PRIMARY KEY, " +
               "entity_id bigint NOT NULL, " +
               // allowing nulls because a template might not have value, and a task might not have a "done/not" setting yet (if unknown)?
               "booleanValue boolean, " +
               //eg, isDone (which would be an entity)
               "attr_type_id bigint not null, " +
               // see "create table RelationToEntity" for comments about dates' meanings.
               "valid_on_date bigint, " +
               "observation_date bigint not null, " +
               "CONSTRAINT valid_attr_type_id FOREIGN KEY (attr_type_id) REFERENCES entity (id), " +
               "CONSTRAINT valid_parent_id FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE, " +
               "CONSTRAINT valid_ba_sorting FOREIGN KEY (entity_id, form_id, id) REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) " +
               "  DEFERRABLE INITIALLY DEFERRED " +
               ") ")
      dbAction("create index boolean_parent_id on BooleanAttribute (entity_id)")
      dbAction("CREATE TRIGGER ba_attribute_sorting_cleanup BEFORE DELETE ON BooleanAttribute " +
               "FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()")

      dbAction("create sequence FileAttributeKeySequence minvalue " + minIdValue)
      dbAction("create table FileAttribute (" +
               // see comment for this column under "create table RelationToEntity", below:
               "form_id smallint DEFAULT " + PostgreSQLDatabase.getAttributeFormId(Controller.FILE_TYPE) +
               "    NOT NULL CHECK (form_id=" + PostgreSQLDatabase.getAttributeFormId(Controller.FILE_TYPE) + "), " +
               "id bigint DEFAULT nextval('FileAttributeKeySequence') PRIMARY KEY, " +
               "entity_id bigint NOT NULL, " +
               //eg, refers to a type like txt: i.e., could be like mime types, extensions, or mac fork info, etc (which would be an entity in any case).
               "attr_type_id bigint NOT NULL, " +
               "description text NOT NULL, " +
               "original_file_date bigint NOT NULL, " +
               "stored_date bigint NOT NULL, " +
               "original_file_path text NOT NULL, " +
               // now that i already wrote this, maybe storing 'readable' is overkill since the system has to read it to store its content. Maybe there's a use.
               "readable boolean not null, " +
               "writable boolean not null, " +
               "executable boolean not null, " +
               //moved to other table: "contents bit varying NOT NULL, " +
               "size bigint NOT NULL, " +
               // this is the md5 hash in hex (just to see if doc has become corrupted; not intended for security/encryption)
               "md5hash char(32) NOT NULL, " +
               "CONSTRAINT valid_attr_type_id FOREIGN KEY (attr_type_id) REFERENCES entity (id), " +
               "CONSTRAINT valid_parent_id FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE, " +
               "CONSTRAINT valid_fa_sorting FOREIGN KEY (entity_id, form_id, id) REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) " +
               "  DEFERRABLE INITIALLY DEFERRED " +
               ") ")
      dbAction("create index file_parent_id on FileAttribute (entity_id)")
      dbAction("CREATE TRIGGER fa_attribute_sorting_cleanup BEFORE DELETE ON FileAttribute " +
               "FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()")
      // about oids and large objects, blobs: here are some reference links (but consider also which version of postgresql is running):
      //  https://duckduckgo.com/?q=postgresql+large+binary+streams
      //  http://www.postgresql.org/docs/9.1/interactive/largeobjects.html
      //  https://wiki.postgresql.org/wiki/BinaryFilesInDB
      //  http://jdbc.postgresql.org/documentation/80/binary-data.html
      //  http://artofsystems.blogspot.com/2008/07/mysql-postgresql-and-blob-streaming.html
      //  http://stackoverflow.com/questions/2069541/postgresql-jdbc-and-streaming-blobs
      //  http://giswiki.hsr.ch/PostgreSQL_-_Binary_Large_Objects
      dbAction("CREATE TABLE FileAttributeContent (" +
               "file_attribute_id bigint PRIMARY KEY, " +
               "contents_oid lo NOT NULL, " +
               "CONSTRAINT valid_fileattr_id FOREIGN KEY (file_attribute_id) REFERENCES fileattribute (id) ON DELETE CASCADE " +
               ")")
      // This trigger exists because otherwise the binary data from large objects doesn't get cleaned up when the related rows are deleted. For details
      // see the links just above (especially the wiki one).
      // (The reason I PUT THE "UPDATE OR" in the "BEFORE UPDATE OR DELETE" is simply: that is how this page's example (at least as of 2016-06-01:
      //    http://www.postgresql.org/docs/current/static/lo.html
      // ...said to do it.
      //Idea: but we still might want more tests around it? and to use "vacuumlo" module, per that same url?
      dbAction("CREATE TRIGGER om_contents_oid_cleanup BEFORE UPDATE OR DELETE ON fileattributecontent " +
               "FOR EACH ROW EXECUTE PROCEDURE lo_manage(contents_oid)")

      dbAction("create sequence TextAttributeKeySequence minvalue " + minIdValue)
      // the entity_id is the key for the entity on which this text info is recorded; for other meanings see comments on
      // Entity.addQuantityAttribute(...).
      // id must be "unique not null" in ANY database used, because it is the primary key.
      dbAction("create table TextAttribute (" +
               // see comment for this column under "create table RelationToEntity", below:
               "form_id smallint DEFAULT " + PostgreSQLDatabase.getAttributeFormId(Controller.TEXT_TYPE) +
               "    NOT NULL CHECK (form_id=" + PostgreSQLDatabase.getAttributeFormId(Controller.TEXT_TYPE) + "), " +
               "id bigint DEFAULT nextval('TextAttributeKeySequence') PRIMARY KEY, " +
               "entity_id bigint NOT NULL, " +
               "textValue text NOT NULL, " +
               //eg, serial number (which would be an entity)
               "attr_type_id bigint not null, " +
               // see "create table RelationToEntity" for comments about dates' meanings.
               "valid_on_date bigint, " +
               "observation_date bigint not null, " +
               "CONSTRAINT valid_attr_type_id FOREIGN KEY (attr_type_id) REFERENCES entity (id), " +
               "CONSTRAINT valid_parent_id FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE, " +
               "CONSTRAINT valid_ta_sorting FOREIGN KEY (entity_id, form_id, id) REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) " +
               "  DEFERRABLE INITIALLY DEFERRED " +
               ") ")
      dbAction("create index text_parent_id on TextAttribute (entity_id)")
      dbAction("CREATE TRIGGER ta_attribute_sorting_cleanup BEFORE DELETE ON TextAttribute " +
               "FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()")

      dbAction("create sequence RelationToEntityKeySequence minvalue " + minIdValue)
      //Example: a relationship between a state and various counties might be set up like this:
      // The state and each county are Entities. A RelationType (which is an Entity with some
      // additional columns) is bi- directional and indicates some kind of containment relationship, for example between
      // state & counties. In the RelationToEntity table there would be a row whose rel_type_id points to the described RelationType,
      // whose entity_id points to the state Entity, and whose entity_id_2 points to a given county Entity. There would be
      // additional rows for each county, varying only in the value in entity_id_2.
      // And example of something non(?)directional would be where the relationship is identical no matter which way you go, like
      // two human acquaintances). The relationship between a state and county is not the same in reverse. Haven't got a good
      // unidirectional example, so maybe it can be eliminated? (Or maybe it would be something where the "child" doesn't "know"
      // the "parent"--like an electron in an atom? -- revu notes or see what Mark Butler thinks.
      // --Luke Call 8/2003.
      dbAction("create table RelationToEntity (" +
               // see comment for this column under "create table RelationToEntity", below:
               "form_id smallint DEFAULT " + PostgreSQLDatabase.getAttributeFormId(Controller.RELATION_TO_ENTITY_TYPE) +
               "    NOT NULL CHECK (form_id=" + PostgreSQLDatabase.getAttributeFormId(Controller.RELATION_TO_ENTITY_TYPE) + "), " +
               //this can be treated like a primary key (with the advantages of being artificial) but the real one is a bit farther down. This one has the
               //slight or irrelevant disadvantage that it artificially limits the # of rows in this table, but it's still a big #.
               "id bigint DEFAULT nextval('RelationToEntityKeySequence') UNIQUE NOT NULL, " +
               //for lookup in RelationType table, eg "has":
               "rel_type_id bigint NOT NULL, " +
               // what is related (see RelationConnection for "related to what" (related_to_entity_id):
               "entity_id bigint NOT NULL, " +
               // entity_id in RelAttr table is related to what other entity(ies):
               "entity_id_2 bigint NOT NULL, " +
               //valid on date can be null (means no info), or 0 (means 'for all time', not 1970 or whatever that was. At least make it a 1 in that case),
               //or the date it first became valid/true:
               "valid_on_date bigint, " +
               //whenever first observed
               "observation_date bigint not null, " +
               "PRIMARY KEY (rel_type_id, entity_id, entity_id_2), " +
               "CONSTRAINT valid_rel_type_id FOREIGN KEY (rel_type_id) REFERENCES RelationType (entity_id) ON DELETE CASCADE, " +
               "CONSTRAINT valid_related_to_entity_id_1 FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE, " +
               "CONSTRAINT valid_related_to_entity_id_2 FOREIGN KEY (entity_id_2) REFERENCES entity (id) ON DELETE CASCADE, " +
               "CONSTRAINT valid_reltoent_sorting FOREIGN KEY (entity_id, form_id, id) REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) " +
               "  DEFERRABLE INITIALLY DEFERRED " +
               ") ")
      dbAction("create index entity_id_1 on RelationToEntity (entity_id)")
      dbAction("create index entity_id_2 on RelationToEntity (entity_id_2)")
      dbAction("CREATE TRIGGER rte_attribute_sorting_cleanup BEFORE DELETE ON RelationToEntity " +
               "FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()")

      // Would rename this sequence to match the table it's used in now, but the cmd "alter sequence relationtogroupkeysequence rename to groupkeysequence;"
      // doesn't rename the name inside the sequence, and keeping the old name is easier for now than deciding whether to do something about that (more info
      // if you search the WWW for "postgresql bug 3619".
      dbAction("create sequence RelationToGroupKeySequence minvalue " + minIdValue)
      // This table is named "grupo" because otherwise some queries (like "drop table group") don't work unless "group" is quoted, which doesn't work
      // with mixed case; but forcing the dropped names to lowercase and quoted also prevented dropping class and entity in the same command, it seemed.
      // Avoiding the word "group" as a table in sql might prevent other errors too.
      dbAction("create table grupo (" +
               "id bigint DEFAULT nextval('RelationToGroupKeySequence') PRIMARY KEY, " +
               "name varchar(" + PostgreSQLDatabase.entityNameLength + ") NOT NULL, " +
               // intended to be a readonly date: the (*java*-style numeric: milliseconds since 1970-1-1 or such) when this row was inserted (ie, when the
               // entity object was created in the db):
               "insertion_date bigint not null, " +
               "allow_mixed_classes boolean NOT NULL " +
               ") ")

      dbAction("create sequence RelationToGroupKeySequence2 minvalue " + minIdValue)
      dbAction("create table RelationToGroup (" +
               // this column is always the same, and exists to enable the integrity constraint which references it, just below
               "form_id smallint DEFAULT " + PostgreSQLDatabase.getAttributeFormId(Controller.RELATION_TO_GROUP_TYPE) +
               "    NOT NULL CHECK (form_id=" + PostgreSQLDatabase.getAttributeFormId(Controller.RELATION_TO_GROUP_TYPE) + "), " +
               //this can be treated like a primary key (with the advantages of being artificial) but the real one is a bit farther down. This one has the
               //slight or irrelevant disadvantage that it artificially limits the # of rows in this table, but it's still a big #.
               "id bigint DEFAULT nextval('RelationToGroupKeySequence2') UNIQUE NOT NULL, " +
               // the entity id of the containing entity whose attribute (subgroup, RTG) this is:
               "entity_id bigint NOT NULL, " +
               "rel_type_id bigint NOT NULL, " +
               "group_id bigint NOT NULL, " +
               //  idea: Should the 2 dates be eliminated? The code is there, including in the parent class, and they might be useful,
               //  maybe no harm while we wait & see.
               // see "create table RelationToEntity" for comments about dates' meanings.
               "valid_on_date bigint, " +
               "observation_date bigint not null, " +
               "PRIMARY KEY (entity_id, rel_type_id, group_id), " +
               "CONSTRAINT valid_reltogrp_entity_id FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE, " +
               "CONSTRAINT valid_reltogrp_rel_type_id FOREIGN KEY (rel_type_id) REFERENCES relationType (entity_id), " +
               "CONSTRAINT valid_reltogrp_group_id FOREIGN KEY (group_id) REFERENCES grupo (id) ON DELETE CASCADE, " +
               "CONSTRAINT valid_reltogrp_sorting FOREIGN KEY (entity_id, form_id, id) REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) " +
               "  DEFERRABLE INITIALLY DEFERRED " +
               ") ")
      dbAction("create index RTG_entity_id on RelationToGroup (entity_id)")
      dbAction("create index RTG_group_id on RelationToGroup (group_id)")
      dbAction("CREATE TRIGGER rtg_attribute_sorting_cleanup BEFORE DELETE ON RelationToGroup " +
               "FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()")

      /* This table maintains a 1-to-many connection between one entity, and many others in a particular group that it contains.
      Will this clarify terms?: the table below is a (1) "relationship table" (aka relationship entity--not an OM entity but at a lower layer) which tracks
      those entities which are part of a particular group.  The nature of the (2) "relation"-ship between that group of entities and the entity that "has"
      them (or other relationtype to them...) is described by the table RelationToGroup, which is instead of a regular old (3) "RelationToEntity" because #3
      just
      relates Entities to other Entities.  Or in other words, #2 (RelationToGroup) has notes about the tie from Entities to groups of Entities,
      where the specific entities in that group are listed in #1 (this table below).  And the type of relation between them (has, contains,
      is acquainted with...?) is in the 4) relationtogroup table's reference to the relationtype table (or its "rel_type_id"). Got it?
      (Good, then let's not confuse things by mentioning that postgresql refers to *every* table (and more?) as a "relation" because that's another
      context altogether, another use of the word.)
      */
      dbAction("create table EntitiesInAGroup (" +
               "group_id bigint NOT NULL" +
               ", entity_id bigint NOT NULL" +
               ", sorting_index bigint not null" +
               // the key is really the group_id + entity_id, and the sorting_index is just in an index so we can cheaply order query results
               // When sorting_index was part of the key there were ongoing various problems because the rest of the system (like reordering results, but
               // probably also other issues) wasn't ready to handle two of the same entity in a group.
               ", PRIMARY KEY (group_id, entity_id)" +
               ", CONSTRAINT valid_group_id FOREIGN KEY (group_id) REFERENCES grupo (id) ON DELETE CASCADE" +
               ", CONSTRAINT valid_entity_id FOREIGN KEY (entity_id) REFERENCES entity (id)" +
               // make it so the sorting_index must also be unique for each group (otherwise we have sorting problems):
               ", constraint noDupSortingIndexes unique (group_id, sorting_index)" +
               ") ")
      dbAction("create index EntitiesInAGroup_id on EntitiesInAGroup (entity_id)")
      dbAction("create index EntitiesInAGroup_sorted on EntitiesInAGroup (group_id, entity_id, sorting_index)")

      dbAction("create sequence ActionKeySequence minvalue " + minIdValue)
      dbAction("create table Action (" +
               "id bigint DEFAULT nextval('ActionKeySequence') PRIMARY KEY, " +
               "class_id bigint NOT NULL, " +
               "name varchar(" + PostgreSQLDatabase.entityNameLength + ") NOT NULL, " +
               "action varchar(" + PostgreSQLDatabase.entityNameLength + ") NOT NULL, " +
               "CONSTRAINT valid_related_to_class_id FOREIGN KEY (class_id) REFERENCES Class (id) ON DELETE CASCADE " +
               ") ")
      dbAction("create index action_class_id on Action (class_id)")

      dbAction("UPDATE om_db_version SET (version) = (" + PostgreSQLDatabase.CURRENT_DB_VERSION + ")")
      commitTrans()
    } catch {
      case e: Exception => throw rollbackWithCatch(e)
    }
  }

  /** Performs automatic database upgrades as required by evolving versions of OneModel.
    *
    * ******MAKE SURE*****:       ...that everything this does is also done in createTables so that createTables is a single reference
    * point for a developer to go read about the database structure, and for testing!  I.e., a newly-created OM instance shouldn't have to be upgraded,
    * because createTables always provides the latest structure in a new system.  This method is just for updating older instances to what is in createTables!
    */
  def doDatabaseUpgradesIfNeeded(): Unit = {
    val versionTableExists: Boolean = doesThisExist("select count(1) from pg_class where relname='om_db_version'")
    if (! versionTableExists) {
      createVersionTable()
    }
    val dbVersion: Int = dbQueryWrapperForOneRow("select version from om_db_version", "Int")(0).get.asInstanceOf[Int]
    if (dbVersion == 0) {
      upgradeDbFrom0to1()
    }
    if (dbVersion == 1) {
      upgradeDbFrom1to2()
    }
    /* NOTE FOR FUTURE METHODS LIKE upgradeDbFrom0to1: methods like this should be designed carefully and very well-tested:
       0) make & test periodic backups of your live data to be safe!
       1) Consider designing it to be idempotent: so multiple runs on a production db (if by some mistake) will no harm (or at least will err out safely).
       2) Could run it against the test db (even though its tables already should have these changes, by being created from scratch), by not yet updating
          the table om_db_version (perhaps by temporarily commenting out the line with
          "UPDATE om_db_version ..." from createTables while running tests).  AND,
       3) Could do a backup, open psql, start a transaction, paste the method's upgrade
          commands there, do manual verifications, then rollback.
       It doesn't seem to make sense to test methods like this with a unit test because the tests are run on a db created as a new
       system, so there is no upgrade to do on a new test, and no known need to call this method except on old systems being upgraded.
       (See also related comment above this doDatabaseUpgradesIfNeeded method.)
      */

    require(dbVersion == PostgreSQLDatabase.CURRENT_DB_VERSION)
  }

  def createVersionTable(): Long = {
    // table has 1 row and 1 column, to say what db version we are on.
    dbAction("create table om_db_version (version integer DEFAULT 1) ")
    dbAction("INSERT INTO om_db_version (version) values (0)")
  }

  private def upgradeDbFrom0to1() = {
    beginTrans()
    try {
      dbAction("ALTER TABLE AttributeSorting DROP CONSTRAINT valid_attribute_form_id")
      dbAction("ALTER TABLE AttributeSorting ADD CONSTRAINT valid_attribute_form_id CHECK (attribute_form_id >= 1 AND attribute_form_id <= 7)")
      createAttributeSortingDeletionTrigger()
      dbAction("ALTER TABLE QuantityAttribute DROP CONSTRAINT valid_qa_sorting")
      dbAction("ALTER TABLE QuantityAttribute ADD CONSTRAINT valid_qa_sorting FOREIGN KEY (entity_id, form_id, id) " +
               "REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) " + "  DEFERRABLE INITIALLY DEFERRED ")
      dbAction("ALTER TABLE DateAttribute DROP CONSTRAINT valid_da_sorting")
      dbAction("ALTER TABLE DateAttribute ADD CONSTRAINT valid_da_sorting FOREIGN KEY (entity_id, form_id, id) " +
               "REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) " + "  DEFERRABLE INITIALLY DEFERRED ")
      dbAction("ALTER TABLE BooleanAttribute DROP CONSTRAINT valid_ba_sorting")
      dbAction("ALTER TABLE BooleanAttribute ADD CONSTRAINT valid_ba_sorting FOREIGN KEY (entity_id, form_id, id) " +
               "REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) " + "  DEFERRABLE INITIALLY DEFERRED " )
      dbAction("ALTER TABLE FileAttribute DROP CONSTRAINT valid_fa_sorting")
      dbAction("ALTER TABLE FileAttribute ADD CONSTRAINT valid_fa_sorting FOREIGN KEY (entity_id, form_id, id) " +
               "REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) " + "  DEFERRABLE INITIALLY DEFERRED ")
      dbAction("ALTER TABLE TextAttribute DROP CONSTRAINT valid_ta_sorting")
      dbAction("ALTER TABLE TextAttribute ADD CONSTRAINT valid_ta_sorting FOREIGN KEY (entity_id, form_id, id) " +
               "REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) " + "  DEFERRABLE INITIALLY DEFERRED ")
      dbAction("ALTER TABLE RelationToEntity DROP CONSTRAINT valid_reltoent_sorting")
      dbAction("ALTER TABLE RelationToEntity ADD CONSTRAINT valid_reltoent_sorting FOREIGN KEY (entity_id, form_id, id) " +
               "REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) " + "  DEFERRABLE INITIALLY DEFERRED ")
      dbAction("ALTER TABLE RelationToGroup DROP CONSTRAINT valid_reltogrp_sorting")
      dbAction("ALTER TABLE RelationToGroup ADD CONSTRAINT valid_reltogrp_sorting FOREIGN KEY (entity_id, form_id, id) " +
               "REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) " + "  DEFERRABLE INITIALLY DEFERRED ")
      dbAction("CREATE TRIGGER qa_attribute_sorting_cleanup BEFORE DELETE ON QuantityAttribute " +
               "FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()")
      dbAction("CREATE TRIGGER da_attribute_sorting_cleanup BEFORE DELETE ON DateAttribute " +
               "FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()")
      dbAction("CREATE TRIGGER ba_attribute_sorting_cleanup BEFORE DELETE ON BooleanAttribute " +
               "FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()")
      dbAction("CREATE TRIGGER fa_attribute_sorting_cleanup BEFORE DELETE ON FileAttribute " +
               "FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()")
      dbAction("CREATE TRIGGER ta_attribute_sorting_cleanup BEFORE DELETE ON TextAttribute " +
               "FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()")
      dbAction("CREATE TRIGGER rte_attribute_sorting_cleanup BEFORE DELETE ON RelationToEntity " +
               "FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()")
      dbAction("CREATE TRIGGER rtg_attribute_sorting_cleanup BEFORE DELETE ON RelationToGroup " +
               "FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()")

      dbAction("UPDATE om_db_version SET (version) = (1)")
    }
    catch {
      case e: Exception => throw rollbackWithCatch(e)
    }
    commitTrans()
  }
  private def upgradeDbFrom1to2() = {
    beginTrans()
    try {
      dbAction("ALTER TABLE class ADD COLUMN create_default_attributes boolean")

      dbAction("UPDATE om_db_version SET (version) = (2)")
    }
    catch {
      case e: Exception => throw rollbackWithCatch(e)
    }
    commitTrans()
  }

  def createAttributeSortingDeletionTrigger(): Long = {
    // Each time an attribute (or rte/rtg) is deleted, the AttributeSorting row should be deleted too, in an enforced way (or it had sorting problems, for one).
    // I.e., an attempt to enforce (with triggers that call this procedure) that the AttributeSorting table's attribute_id value is found
    // in *one of the* 7 attribute tables' id column,  Doing it in application code is not as simple or as reliable as doing it at the DDL level.
    val sql = "CREATE OR REPLACE FUNCTION attribute_sorting_cleanup() RETURNS trigger AS $attribute_sorting_cleanup$ " +
              "  BEGIN" +
              // (OLD is a special PL/pgsql variable of type RECORD, which contains the attribute row before the deletion.)
              "    DELETE FROM AttributeSorting WHERE entity_id=OLD.entity_id and attribute_form_id=OLD.form_id and attribute_id=OLD.id; " +
              "    RETURN OLD; " +
              "  END;" +
              "$attribute_sorting_cleanup$ LANGUAGE plpgsql;"
    dbAction(sql, skipCheckForBadSqlIn = true)
  }

  def findAllEntityIdsByName(inName: String, caseSensitive: Boolean = false): Option[List[Long]] = {
    // idea: see if queries like this are using the expected index (run & ck the query plan). Tests around that, for benefit of future dbs? Or, just wait for
    // a performance issue then look at it?
    val sql = "select id from entity where " +
              (if (!showAllArchivedEntities) {
                "(not archived) and "
              } else {
                ""
              }) +
              {
                if (caseSensitive) "name = '" + inName + "'"
                else "lower(name) = lower('" + inName + "'" + ")"
              }
    val rows = dbQuery(sql, "Long")

    if (rows.isEmpty) None
    else {
      var results: List[Long] = Nil
      for (row <- rows) {
        results = row(0).get.asInstanceOf[Long] :: results
      }
      Some(results.reverse)
    }
  }

  // See comment in ImportExport.processUriContent method which uses it, about where the code should really go. Not sure if that idea includes this
  // method or not.
  def findFIRSTClassIdByName(inName: String, caseSensitive: Boolean = false): Option[Long] = {
    // idea: see if queries like this are using the expected index (run & ck the query plan). Tests around that, for benefit of future dbs? Or, just wait for
    // a performance issue then look at it?
    val nameClause = {
      if (caseSensitive) "name = '" + inName + "'"
      else "lower(name) = lower('" + inName + "'" + ")"
    }
    val sql = "select id from class where " + nameClause + " order by id limit 1"
    val rows = dbQuery(sql, "Long")

    if (rows.isEmpty) None
    else {
      var results: List[Long] = Nil
      for (row <- rows) {
        results = row(0).get.asInstanceOf[Long] :: results
      }
      if (results.size > 1) throw new OmDatabaseException("Expected 1 row (wanted just the first one), found " + results.size + " rows.")
      Some(results.head)
    }
  }

  /** Case-insensitive. */
  def findEntityOnlyIdsByName(nameIn: String): Option[List[Long]] = {
    // idea: see if queries like this are using the expected index (run & ck the query plan). Tests around that, for benefit of future dbs? Or, just wait for
    // a performance issue then look at it?
    val rows = dbQuery("select id from entity where " +
                       (if (!showAllArchivedEntities) {
                         "(not archived) and "
                       } else {
                         ""
                       }) +
                       "lower(name) = lower('" + nameIn + "') " + limitToEntitiesOnly(ENTITY_ONLY_SELECT_PART),
                       "Long")
    if (rows.isEmpty) None
    else {
      var results: List[Long] = Nil
      for (row <- rows) {
        results = row(0).get.asInstanceOf[Long] :: results
      }
      Some(results.reverse)
    }
  }

  /** @param searchStringIn is case-insensitive.
    * @param stopAfterAnyFound is to prevent a serious performance problem when searching for the default entity at startup, and that default entity
    *                          eventually links to 1000's of others.  Alternatives included specifying a different levelsRemaining parameter in that
    *                          case, or not following any RelationToEntity links (which defeats the ability to organize the preferences in a hierarchy),
    *                          or flagging certain ones to skip by marking them as a preference (not a link to follow in the preferences hierarchy), but
    *                          those all seemed more complicated.
    * */
  def findContainedEntityIds(resultsInOut: mutable.TreeSet[Long], fromEntityIdIn: Long, searchStringIn: String,
                             levelsRemaining: Int = 20, stopAfterAnyFound: Boolean = true): mutable.TreeSet[Long] = {
    // Idea for optimizing: don't re-traverse dup ones (eg, circular links or entities in same two places).  But that has other complexities: see
    // comments on ImportExport.exportItsChildrenToHtmlFiles for more info.  But since we are limiting the # of levels total, it might not matter anyway
    // (ie, probably the current code is not optimized but is simpler and good enough for now).

    // Idea: could do regexes instead of string matching, like we have elsewhere (& are now, for TextAttributes below)? If so, put similar text in the prompt
    // (see Controller.findExistingObjectByText, clarify in the method names/docs that we are doing regexes, & methods getMatchingEntities, getMatchingGroups.

    if (levelsRemaining <= 0 || (stopAfterAnyFound && resultsInOut.nonEmpty)) {
      // do nothing: get out.
    } else {
      val sql = "select rte.entity_id_2, e.name from entity e, RelationToEntity rte where rte.entity_id=" + fromEntityIdIn +
                " and rte.entity_id_2=e.id " +
                (if (!showAllArchivedEntities) {
                  "and not e.archived"
                } else {
                  ""
                })
      val relatedEntityIdRows = dbQuery(sql, "Long,String")
      for (row <- relatedEntityIdRows) {
        val id: Long = row(0).get.asInstanceOf[Long]
        val name = row(1).get.asInstanceOf[String]
        if (name.toLowerCase.contains(searchStringIn.toLowerCase)) {
          // have to do the name check here because we need to traverse all contained entities, so we need all those back from the sql, not just name matches.
          resultsInOut.add(id)
        }
        findContainedEntityIds(resultsInOut, id, searchStringIn, levelsRemaining - 1, stopAfterAnyFound)
      }
      if (! (stopAfterAnyFound && resultsInOut.nonEmpty)) {
        val sql2 = "select eiag.entity_id, e.name from RelationToGroup rtg, EntitiesInAGroup eiag, entity e where rtg.entity_id=" + fromEntityIdIn +
                   " and rtg.group_id=eiag.group_id and eiag.entity_id=e.id" +
                   (if (!showAllArchivedEntities) {
                     " and not e.archived"
                   } else {
                     ""
                   })
        val entitiesInGroups = dbQuery(sql2, "Long,String")
        for (row <- entitiesInGroups) {
          val id: Long = row(0).get.asInstanceOf[Long]
          val name = row(1).get.asInstanceOf[String]
          if (name.toLowerCase.contains(searchStringIn.toLowerCase)) {
            // have to do the name check here because we need to traverse all contained entities, so we need all those back from the sql, not just name matches.
            resultsInOut.add(id)
          }
          findContainedEntityIds(resultsInOut, id, searchStringIn, levelsRemaining - 1, stopAfterAnyFound)
        }
      }
      // this part is doing a regex now:
      if (! (stopAfterAnyFound && resultsInOut.nonEmpty)) {
        val sql3 = "select ta.id from textattribute ta, entity e where entity_id=e.id" +
                   (if (!showAllArchivedEntities) {
                     " and (not e.archived)"
                   } else {
                     ""
                   }) +
                   " and entity_id=" + fromEntityIdIn +
                   " and textValue ~* '" + searchStringIn + "'"
        val textAttributes: List[Array[Option[Any]]] = dbQuery(sql3, "Long")
        if (textAttributes.nonEmpty) {
          resultsInOut.add(fromEntityIdIn)
        }
      }
    }
    resultsInOut
  }

  /** Creates data that must exist in a base system, and which is not re-created in an existing system.  If this data is deleted, the system might not work.  
    */
  def createBaseData() {
    // idea: what tests are best, around this, vs. simply being careful in upgrade scripts?
    val ids: Option[List[Long]] = findEntityOnlyIdsByName(PostgreSQLDatabase.systemEntityName)
    // will probably have to change the next line when things grow/change, and say, we're doing upgrades not always a new system:
    require(ids.isEmpty)

    // public=false, guessing at best value, since the world wants your modeled info, not details about your system internals (which might be...unique &
    // personal somehow)?:
    val systemEntityId = createEntity(PostgreSQLDatabase.systemEntityName, isPublicIn = Some(false))

    val existenceEntityId = createEntity("existence", isPublicIn = Some(false))
    //idea: as probably mentioned elsewhere, this "BI" (and other strings?) should be replaced with a constant somewhere (or enum?)!
    val hasRelTypeId = createRelationType(PostgreSQLDatabase.theHASrelationTypeName, PostgreSQLDatabase.theIsHadByReverseName, "BI")
    createRelationToEntity(hasRelTypeId, systemEntityId, existenceEntityId, Some(System.currentTimeMillis()), System.currentTimeMillis())

    val editorInfoEntityId = createEntity(PostgreSQLDatabase.EDITOR_INFO_ENTITY_NAME, isPublicIn = Some(false))
    createRelationToEntity(hasRelTypeId, systemEntityId, editorInfoEntityId, Some(System.currentTimeMillis()), System.currentTimeMillis())
    val textEditorInfoEntityId = createEntity(PostgreSQLDatabase.TEXT_EDITOR_INFO_ENTITY_NAME, isPublicIn = Some(false))
    createRelationToEntity(hasRelTypeId, editorInfoEntityId, textEditorInfoEntityId, Some(System.currentTimeMillis()), System.currentTimeMillis())
    val textEditorCommandAttributeTypeId = createEntity(PostgreSQLDatabase.TEXT_EDITOR_COMMAND_ATTRIBUTE_TYPE_NAME, isPublicIn = Some(false))
    createRelationToEntity(hasRelTypeId, textEditorInfoEntityId, textEditorCommandAttributeTypeId, Some(System.currentTimeMillis()), System.currentTimeMillis())
    val editorCommand: String = {
      if (Controller.isWindows) "notepad"
      else "vi"
    }
    createTextAttribute(textEditorInfoEntityId, textEditorCommandAttributeTypeId, editorCommand, Some(System.currentTimeMillis()))


    // the intent of this group is user convenience: the app shouldn't rely on this group to find classDefiningEntities (templates), but use the relevant table.
    // idea: REALLY, this should probably be replaced with a query to the class table: so, when queries as menu options are part of the OM
    // features, put them all there instead.
    // It is set to allowMixedClassesInGroup just because no current known reason not to, will be interesting to see what comes of it.
    createGroupAndRelationToGroup(systemEntityId, hasRelTypeId, PostgreSQLDatabase.classTemplateEntityGroupName, allowMixedClassesInGroupIn = true,
                                  Some(System.currentTimeMillis()), System.currentTimeMillis(), None, callerManagesTransactionsIn = false)

    // NOTICE: code should not rely on this name, but on data in the tables.
    /*val (classId, entityId) = */ createClassAndItsTemplateEntity("person-template")
  }

  /** Returns the classId and entityId, in a tuple. */
  def createClassAndItsTemplateEntity(inName: String): (Long, Long) = {
    // The name doesn't have to be the same on the entity and the template class, but why not for now.
    val name: String = escapeQuotesEtc(inName)
    if (name == null || name.length == 0) throw new OmDatabaseException("Name must have a value.")
    val classId: Long = getNewKey("ClassKeySequence")
    val entityId: Long = getNewKey("EntityKeySequence")
    beginTrans()
    try {
      // Start the entity w/ a NULL class_id so that it can be inserted w/o the class present, then update it afterward; constraints complain otherwise.
      // Idea: instead of doing in 3 steps, could specify 'deferred' on the 'not null'
      // constraint?: (see file:///usr/share/doc/postgresql-doc-9.1/html/sql-createtable.html).
      dbAction("INSERT INTO Entity (id, insertion_date, name, class_id) VALUES (" + entityId + "," + System.currentTimeMillis() + ",'" + name + "', NULL)")
      dbAction("INSERT INTO Class (id, name, defining_entity_id) VALUES (" + classId + ",'" + name + "', " + entityId + ")")
      dbAction("update Entity set (class_id) = (" + classId + ") where id=" + entityId)
    } catch {
      case e: Exception => throw rollbackWithCatch(e)
    }
    commitTrans()

    val classGroupId = getSystemEntitysClassGroupId
    if (classGroupId.isDefined) {
      addEntityToGroup(classGroupId.get, entityId)
    }

    (classId, entityId)
  }

  /** Returns the id of a specific group under the system entity.  This group is the one that contains class-defining (template) entities. */
  def getSystemEntitysClassGroupId: Option[Long] = {
    val systemEntityId: Long = getSystemEntityId

    // idea: maybe this stuff would be less breakable by the user if we put this kind of info in some system table
    // instead of in this group. (See also method createDefaultData).  Or maybe it doesn't matter, since it's just a user convenience. Hmm.
    val classTemplateGroupId = findRelationToAndGroup_OnEntity(systemEntityId, Some(PostgreSQLDatabase.classTemplateEntityGroupName))._3
    if (classTemplateGroupId.isEmpty) {
      // no exception thrown here because really this group is a convenience for the user to see things, not a requirement. Maybe a user message would be best:
      // Idea:: BAD SMELL! The UI should do all UI communication, no?
      System.err.println("Unable to find, from the entity " + PostgreSQLDatabase.systemEntityName + "(" + systemEntityId + "), " +
                         "any connection to its expected contained group " +
                         PostgreSQLDatabase.classTemplateEntityGroupName + ".  If it was deleted, it could be replaced if you want the convenience of finding" +
                         " template " +
                         "entities in it.")
    }
    classTemplateGroupId
  }

  def deleteClassAndItsTemplateEntity(inClassId: Long) {
    beginTrans()
    try {
      val templateEntityId: Long = getClassData(inClassId)(1).get.asInstanceOf[Long]
      val classGroupId = getSystemEntitysClassGroupId
      if (classGroupId.isDefined) {
        removeEntityFromGroup(classGroupId.get, templateEntityId, callerManagesTransactionsIn = true)
      }
      updateEntitysClass(templateEntityId, None, callerManagesTransactions = true)
      deleteObjectById("class", inClassId, callerManagesTransactions = true)
      deleteObjectById(Controller.ENTITY_TYPE, templateEntityId, callerManagesTransactions = true)
    } catch {
      case e: Exception => throw rollbackWithCatch(e)
    }
    commitTrans()
  }

  /** Returns at most 1 row's info (id, relationTypeId, groupId), and a boolean indicating if more were available.  If 0 rows are found, returns (None,false),
    * so this expects the caller
    * to know there is only one or deal with the None.
    */
  def findRelationToAndGroup_OnEntity(inEntityId: Long, inGroupName: Option[String] = None): (Option[Long], Option[Long], Option[Long], Boolean) = {
    val nameCondition = if (inGroupName.isDefined) {
      val name = escapeQuotesEtc(inGroupName.get)
      "g.name='" + name + "'"
    } else
      "true"

    // "limit 2", so we know and can return whether more were available:
    val rows = dbQuery("select rtg.id, rtg.rel_type_id, g.id from relationtogroup rtg, grupo g where rtg.group_id=g.id and rtg.entity_id=" + inEntityId +
                       " and " + nameCondition + " order by rtg.id limit 2", "Long,Long,Long")
    // there could be none found, or more than one, but:
    if (rows.isEmpty)
      (None, None, None, false)
    else {
      val row = rows.head
      val id: Option[Long] = Some(row(0).get.asInstanceOf[Long])
      val relTypeId: Option[Long] = Some(row(1).get.asInstanceOf[Long])
      val groupId: Option[Long] = Some(row(2).get.asInstanceOf[Long])
      (id, relTypeId, groupId, rows.size > 1)
    }
  }

  /**
   * @return the id of the new RTE
   */
  def addHASRelationToEntity(fromEntityIdIn: Long, toEntityIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long,
                             sortingIndexIn: Option[Long] = None): RelationToEntity = {
    val relationTypeId = findRelationType(PostgreSQLDatabase.theHASrelationTypeName, Some(1))(0)
    val newRte = createRelationToEntity(relationTypeId, fromEntityIdIn, toEntityIdIn, validOnDateIn, observationDateIn, sortingIndexIn)
    newRte
  }

  /** Returns at most 1 id, and a boolean indicating if more were available.  If 0 rows are found, returns (None,false), so this expects the caller
    * to know there is only one or deal with the None.
    */
  def findRelationType(inTypeName: String, expectedRows: Option[Int] = Some(1)): Array[Long] = {
    val name = escapeQuotesEtc(inTypeName)
    val rows = dbQuery("select entity_id from entity e, relationtype rt where e.id=rt.entity_id and name='" + name + "' order by id limit 2", "Long")
    if (expectedRows.isDefined) {
      val count = rows.size
      if (count != expectedRows.get) throw new OmDatabaseException("Found " + count + " rows instead of expected " + expectedRows)
    }
    // there could be none found, or more than one, but
    val finalResult = new Array[Long](rows.size)
    var index = 0
    for (row <- rows) {
      val id: Option[Long] = Some(row(0).get.asInstanceOf[Long])
      finalResult(index) = id.get
      index += 1
    }
    finalResult
  }

  /** Indicates whether the database setup has been done. */
  def modelTablesExist: Boolean = doesThisExist("select count(1) from pg_class where relname='entity'")

  /** Used, for example, when test code is finished with its test data. Be careful. */
  def destroyTables() {
    PostgreSQLDatabase.destroyTables_helper(mConn)
  }

  /**
   * Saves data for a quantity attribute for a Entity (i.e., "6 inches length").<br>
   * parentIdIn is the key of the Entity for which the info is being saved.<br>
   * inUnitId represents a Entity; indicates the unit for this quantity (i.e., liters or inches).<br>
   * inNumber represents "how many" of the given unit.<br>
   * attrTypeIdIn represents the attribute type and also is a Entity (i.e., "volume" or "length")<br>
   * validOnDateIn represents the date on which this began to be true (seems it could match the observation date if needed,
   * or guess when it was definitely true);
   * NULL means unknown, 0 means it is asserted true for all time. inObservationDate is the date the fact was observed. <br>
   * <br>
   * We store the dates in
   * postgresql (at least) as bigint which should be the same size as a java long, with the understanding that we are
   * talking about java-style dates here; it is my understanding that such long's can also be negative to represent
   * dates long before 1970, or positive for dates long after 1970. <br>
   * <br>
   * In the case of inNumber, note
   * that the postgresql docs give some warnings about the precision of its real and "double precision" types. Given those
   * warnings and the fact that I haven't investigated carefully (as of 9/2002) how the data will be saved and read
   * between the java float type and the postgresql types, I am using "double precision" as the postgresql data type,
   * as a guess to try to lose as
   * little information as possible, and I'm making this note to you the reader, so that if you care about the exactness
   * of the data you can do some research and let us know what you find.
   * <p/>
   * Re dates' meanings: see usage notes elsewhere in code (like inside createTables).
   */
  def createQuantityAttribute(inParentId: Long, inAttrTypeId: Long, inUnitId: Long, inNumber: Float, inValidOnDate: Option[Long],
                              inObservationDate: Long, callerManagesTransactionsIn: Boolean = false, sortingIndexIn: Option[Long] = None): /*id*/ Long = {
    if (!callerManagesTransactionsIn) beginTrans()
    var id: Long = 0L
    try {
      id = getNewKey("QuantityAttributeKeySequence")
      addAttributeSortingRow(inParentId, PostgreSQLDatabase.getAttributeFormId(Controller.QUANTITY_TYPE), id, sortingIndexIn)
      dbAction("insert into QuantityAttribute (id, entity_id, unit_id, quantity_number, attr_type_id, valid_on_date, observation_date) " +
               "values (" + id + "," + inParentId + "," + inUnitId + "," + inNumber + "," + inAttrTypeId + "," +
               (if (inValidOnDate.isEmpty) "NULL" else inValidOnDate.get) + "," + inObservationDate + ")")
    }
    catch {
      case e: Exception =>
        if (!callerManagesTransactionsIn) rollbackTrans()
        throw e
    }
    if (!callerManagesTransactionsIn) commitTrans()
    id
  }

  def escapeQuotesEtc(s: String): String = {
    PostgreSQLDatabase.escapeQuotesEtc(s)
  }

  def checkForBadSql(s: String) {
    PostgreSQLDatabase.checkForBadSql(s)
  }

  def updateQuantityAttribute(inId: Long, inParentId: Long, inAttrTypeId: Long, inUnitId: Long, inNumber: Float, inValidOnDate: Option[Long],
                              inObservationDate: Long) {
    // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
    // in memory when the db updates, and the behavior gets weird.
    dbAction("update QuantityAttribute set (unit_id, quantity_number, attr_type_id, valid_on_date, observation_date) = (" + inUnitId + "," +
             "" + inNumber + "," + inAttrTypeId + "," + (if (inValidOnDate.isEmpty) "NULL" else inValidOnDate.get) + "," +
             "" + inObservationDate + ") where id=" + inId + " and  entity_id=" + inParentId)
  }

  def updateTextAttribute(inId: Long, inParentId: Long, inAttrTypeId: Long, inText: String, inValidOnDate: Option[Long], inObservationDate: Long) {
    val text: String = escapeQuotesEtc(inText)
    // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
    // in memory when the db updates, and the behavior gets weird.
    dbAction("update TextAttribute set (textValue, attr_type_id, valid_on_date, observation_date) = ('" + text + "'," + inAttrTypeId + "," +
             "" + (if (inValidOnDate.isEmpty) "NULL" else inValidOnDate.get) + "," + inObservationDate + ") where id=" + inId + " and  " +
             "entity_id=" + inParentId)
  }

  def updateDateAttribute(inId: Long, inParentId: Long, inDate: Long, inAttrTypeId: Long) {
    // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
    // in memory when the db updates, and the behavior gets weird.
    dbAction("update DateAttribute set (date, attr_type_id) = (" + inDate + "," + inAttrTypeId + ") where id=" + inId + " and  " +
             "entity_id=" + inParentId)
  }

  def updateBooleanAttribute(inId: Long, inParentId: Long, inAttrTypeId: Long, inBoolean: Boolean, inValidOnDate: Option[Long], inObservationDate: Long) {
    // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
    // in memory when the db updates, and the behavior gets weird.
    dbAction("update BooleanAttribute set (booleanValue, attr_type_id, valid_on_date, observation_date) = (" + inBoolean + "," + inAttrTypeId + "," +
             "" + (if (inValidOnDate.isEmpty) "NULL" else inValidOnDate.get) + "," + inObservationDate + ") where id=" + inId + " and  " +
             "entity_id=" + inParentId)
  }

  // We don't update the dates, path, size, hash because we set those based on the file's own timestamp, path current date,
  // & contents when it is written. So the only
  // point to having an update method might be the attribute type & description.
  // AND THAT: The validOnDate for a file attr shouldn't ever be None/NULL like with other attrs, because it is the file date in the filesystem before it was
  // read into OM.
  def updateFileAttribute(inId: Long, inParentId: Long, inAttrTypeId: Long, inDescription: String) {
    // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
    // in memory when the db updates, and the behavior gets weird.
    dbAction("update FileAttribute set (description, attr_type_id) = ('" + inDescription + "'," + inAttrTypeId + ")" +
             " where id=" + inId + " and entity_id=" + inParentId)
  }

  // first take on this: might have a use for it later.  It's tested, and didn't delete, but none known now. Remove?
  def updateFileAttribute(inId: Long, inParentId: Long, inAttrTypeId: Long, inDescription: String, originalFileDateIn: Long, storedDateIn: Long,
                          originalFilePathIn: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, sizeIn: Long, md5hashIn: String) {
    // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
    // in memory when the db updates, and the behavior gets weird.
    dbAction("update FileAttribute set " +
             " (description, attr_type_id, original_file_date, stored_date, original_file_path, readable, writable, executable, size, md5hash) =" +
             " ('" + inDescription + "'," + inAttrTypeId + "," + originalFileDateIn + "," + storedDateIn + ",'" + originalFilePathIn + "'," +
             " " + readableIn + "," + writableIn + "," + executableIn + "," +
             " " + sizeIn + "," +
             " '" + md5hashIn + "')" +
             " where id=" + inId + " and entity_id=" + inParentId)
  }

  def updateEntityOnlyName(inId: Long, nameIn: String) {
    val name: String = escapeQuotesEtc(nameIn)
    dbAction("update Entity set (name) = ('" + name + "') where id=" + inId)
  }

  def updateEntityOnlyPublicStatus(inId: Long, value: Option[Boolean]) {
    dbAction("update Entity set (public) = (" +
             (if (value.isEmpty) "NULL" else if (value.get) "true" else "false") +
             ") where id=" + inId)
  }

  def updateClassAndTemplateEntityName(classIdIn: Long, name: String): Long = {
    var entityId: Long = 0
    beginTrans()
    try {
      updateClassName(classIdIn, name)
      entityId = new EntityClass(this, classIdIn).getTemplateEntityId
      updateEntityOnlyName(entityId, name)
    }
    catch {
      case e: Exception => throw rollbackWithCatch(e)
    }
    commitTrans()
    entityId
  }

  def updateClassName(inId: Long, nameIn: String) {
    val name: String = escapeQuotesEtc(nameIn)
    dbAction("update class set (name) = ('" + name + "') where id=" + inId)
  }

  def updateEntitysClass(entityId: Long, classId: Option[Long], callerManagesTransactions: Boolean = false) {
    if (!callerManagesTransactions) beginTrans()
    dbAction("update Entity set (class_id) = (" +
             (if (classId.isEmpty) "NULL" else classId.get) +
             ") where id=" + entityId)
    val groupIds = dbQuery("select group_id from EntitiesInAGroup where entity_id=" + entityId, "Long")
    for (row <- groupIds) {
      val groupId = row(0).get.asInstanceOf[Long]
      val mixedClassesAllowed: Boolean = areMixedClassesAllowed(groupId)
      if ((!mixedClassesAllowed) && hasMixedClasses(groupId)) {
        throw rollbackWithCatch(new OmDatabaseException(PostgreSQLDatabase.MIXED_CLASSES_EXCEPTION))
      }
    }
    if (!callerManagesTransactions) commitTrans()
  }

  def updateRelationType(inId: Long, inName: String, inNameInReverseDirection: String, inDirectionality: String) {
    require(inName != null)
    require(inName.length > 0)
    require(inNameInReverseDirection != null)
    require(inNameInReverseDirection.length > 0)
    require(inDirectionality != null)
    require(inDirectionality.length > 0)
    val nameInReverseDirection: String = escapeQuotesEtc(inNameInReverseDirection)
    val name: String = escapeQuotesEtc(inName)
    val directionality: String = escapeQuotesEtc(inDirectionality)
    beginTrans()
    try {
      dbAction("update Entity set (name) = ('" + name + "') where id=" + inId)
      dbAction("update RelationType set (name_in_reverse_direction, directionality) = ('" + nameInReverseDirection + "', " +
               "'" + directionality + "') where entity_id=" + inId)
    } catch {
      case e: Exception => throw rollbackWithCatch(e)
    }
    commitTrans()
  }

  /** Re dates' meanings: see usage notes elsewhere in code (like inside createTables). */
  def createTextAttribute(parentIdIn: Long, attrTypeIdIn: Long, inText: String, validOnDateIn: Option[Long] = None,
                          observationDateIn: Long = System.currentTimeMillis(), callerManagesTransactionsIn: Boolean = false,
                          sortingIndexIn: Option[Long] = None): /*id*/ Long = {
    val text: String = escapeQuotesEtc(inText)
    val id: Long = getNewKey("TextAttributeKeySequence")
    if (!callerManagesTransactionsIn) beginTrans()
    try {
      addAttributeSortingRow(parentIdIn, PostgreSQLDatabase.getAttributeFormId(Controller.TEXT_TYPE), id, sortingIndexIn)
      dbAction("insert into TextAttribute (id, entity_id, textvalue, attr_type_id, valid_on_date, observation_date) " +
               "values (" + id + "," + parentIdIn + ",'" + text + "'," + attrTypeIdIn + "," +
               "" + (if (validOnDateIn.isEmpty) "NULL" else validOnDateIn.get) + "," + observationDateIn + ")")
    }
    catch {
      case e: Exception =>
        if (!callerManagesTransactionsIn) rollbackTrans()
        throw e
    }
    if (!callerManagesTransactionsIn) commitTrans()
    id
  }

  def createDateAttribute(parentIdIn: Long, attrTypeIdIn: Long, dateIn: Long, sortingIndexIn: Option[Long] = None): /*id*/ Long = {
    val id: Long = getNewKey("DateAttributeKeySequence")
    beginTrans()
    try {
      addAttributeSortingRow(parentIdIn, PostgreSQLDatabase.getAttributeFormId(Controller.DATE_TYPE), id, sortingIndexIn)
      dbAction("insert into DateAttribute (id, entity_id, attr_type_id, date) " +
               "values (" + id + "," + parentIdIn + ",'" + attrTypeIdIn + "'," + dateIn + ")")
    }
    catch {
      case e: Exception => throw rollbackWithCatch(e)
    }
    commitTrans()
    id
  }

  def createBooleanAttribute(parentIdIn: Long, attrTypeIdIn: Long, booleanIn: Boolean, validOnDateIn: Option[Long], observationDateIn: Long,
                             sortingIndexIn: Option[Long] = None): /*id*/ Long = {
    val id: Long = getNewKey("BooleanAttributeKeySequence")
    beginTrans()
    try {
      addAttributeSortingRow(parentIdIn, PostgreSQLDatabase.getAttributeFormId(Controller.BOOLEAN_TYPE), id, sortingIndexIn)
      dbAction("insert into BooleanAttribute (id, entity_id, booleanvalue, attr_type_id, valid_on_date, observation_date) " +
               "values (" + id + "," + parentIdIn + ",'" + booleanIn + "'," + attrTypeIdIn + "," +
               "" + (if (validOnDateIn.isEmpty) "NULL" else validOnDateIn.get) + "," + observationDateIn + ")")
    }
    catch {
      case e: Exception => throw rollbackWithCatch(e)
    }
    commitTrans()
    id
  }

  def createFileAttribute(parentIdIn: Long, attrTypeIdIn: Long, descriptionIn: String, originalFileDateIn: Long, storedDateIn: Long,
                          originalFilePathIn: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, sizeIn: Long,
                          md5hashIn: String, inputStreamIn: java.io.FileInputStream, sortingIndexIn: Option[Long] = None): /*id*/ Long = {
    val description: String = escapeQuotesEtc(descriptionIn)
    // (Next 2 for completeness but there shouldn't ever be a problem if other code is correct.)
    val originalFilePath: String = escapeQuotesEtc(originalFilePathIn)
    // Escaping the md5hash string shouldn't ever matter, but security is more important than the md5hash:
    val md5hash: String = escapeQuotesEtc(md5hashIn)
    var obj: LargeObject = null
    var id: Long = 0
    try {
      id = getNewKey("FileAttributeKeySequence")
      beginTrans()
      addAttributeSortingRow(parentIdIn, PostgreSQLDatabase.getAttributeFormId(Controller.FILE_TYPE), id, sortingIndexIn)
      dbAction("insert into FileAttribute (id, entity_id, attr_type_id, description, original_file_date, stored_date, original_file_path, readable, writable," +
               " executable, size, md5hash)" +
               " values (" + id + "," + parentIdIn + "," + attrTypeIdIn + ",'" + description + "'," + originalFileDateIn + "," + storedDateIn + "," +
               " '" + originalFilePath + "', " + readableIn + ", " + writableIn + ", " + executableIn + ", " + sizeIn + ",'" + md5hash + "')")
      // from the example at:   http://jdbc.postgresql.org/documentation/80/binary-data.html & info
      // at http://jdbc.postgresql.org/documentation/publicapi/org/postgresql/largeobject/LargeObjectManager.html & its links.
      val lobjManager: LargeObjectManager = mConn.asInstanceOf[org.postgresql.PGConnection].getLargeObjectAPI
      val oid: Long = lobjManager.createLO()
      obj = lobjManager.open(oid, LargeObjectManager.WRITE)
      val buffer = new Array[Byte](2048)
      var numBytesRead = 0
      var total: Long = 0
      @tailrec
      def saveFileToDb() {
        numBytesRead = inputStreamIn.read(buffer)
        // (intentional style violation, for readability):
        //noinspection ScalaUselessExpression
        if (numBytesRead == -1) Unit
        else {
          // (just once by a subclass is enough to mess w/ the md5sum for testing:)
          if (total == 0) damageBuffer(buffer)

          obj.write(buffer, 0, numBytesRead)
          total += numBytesRead
          saveFileToDb()
        }
      }
      saveFileToDb()
      if (total != sizeIn) {
        throw new OmDatabaseException("Transferred " + total + " bytes instead of " + sizeIn + "??")
      }
      dbAction("INSERT INTO FileAttributeContent (file_attribute_id, contents_oid) VALUES (" + id + "," + oid + ")")

      val (success, errMsgOption) = verifyFileAttributeContentIntegrity(id)
      if (!success) {
        throw new OmFileTransferException("Failure to successfully upload file content: " + errMsgOption.getOrElse("(verification provided no error message? " +
                                                                                                                   "how?)"))
      }
      commitTrans()
      id
    } catch {
      case e: Exception => throw rollbackWithCatch(e)
    } finally {
      if (obj != null)
        try {
          obj.close()
        } catch {
          case e: Exception =>
          // not sure why this fails sometimes, if it's a bad thing or not, but for now not going to be stuck on it.
          // idea: look at the source code.
        }
    }
  }

  /** Re dates' meanings: see usage notes elsewhere in code (like inside createTables). */
  def createRelationToEntity(inRelationTypeId: Long, inEntityId1: Long, inEntityId2: Long, inValidOnDate: Option[Long], inObservationDate: Long,
                             sortingIndexIn: Option[Long] = None, callerManagesTransactionsIn: Boolean = false): RelationToEntity = {
    val rteId: Long = getNewKey("RelationToEntityKeySequence")
    if (!callerManagesTransactionsIn) beginTrans()
    try {
      addAttributeSortingRow(inEntityId1, PostgreSQLDatabase.getAttributeFormId(Controller.RELATION_TO_ENTITY_TYPE), rteId, sortingIndexIn)
      dbAction("INSERT INTO RelationToEntity (id, rel_type_id, entity_id, entity_id_2, valid_on_date, observation_date) " +
               "VALUES (" + rteId + "," + inRelationTypeId + "," + inEntityId1 + ", " + inEntityId2 + ", " +
               "" + (if (inValidOnDate.isEmpty) "NULL" else inValidOnDate.get) + "," + inObservationDate + ")")
    }
    catch {
      case e: Exception =>
        if (!callerManagesTransactionsIn) rollbackTrans()
        throw e
    }
    if (!callerManagesTransactionsIn) commitTrans()
    new RelationToEntity(this, rteId, inRelationTypeId, inEntityId1, inEntityId2)
  }

  /** Re dates' meanings: see usage notes elsewhere in code (like inside createTables). */
  def updateRelationToEntity(oldRelationTypeIdIn: Long, inEntityId1: Long, inEntityId2: Long,
                             newRelationTypeIdIn: Long, inValidOnDate: Option[Long], inObservationDate: Long) {
    // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
    // in memory when the db updates, and the behavior gets weird.
    dbAction("UPDATE RelationToEntity SET (rel_type_id, valid_on_date, observation_date)" +
             " = (" + newRelationTypeIdIn + "," + (if (inValidOnDate.isEmpty) "NULL" else inValidOnDate.get) + "," + inObservationDate + ")" +
             " where rel_type_id=" + oldRelationTypeIdIn + " and entity_id=" + inEntityId1 + " and entity_id_2=" + inEntityId2)
  }

  /**
   * @param sortingIndexIn Used because it seems handy (as done in calls to other move methods) to keep it in case one moves many entries: they stay in order.
   * @return the new RelationToEntity
   */
  def moveRelationToEntity(relationToEntityIdIn: Long, newContainingEntityIdIn: Long, sortingIndexIn: Long): RelationToEntity = {
    beginTrans()
    try {
      val rteData: Array[Option[Any]] = getAllRelationToEntityDataById(relationToEntityIdIn)
      val oldRteRelType: Long = rteData(2).get.asInstanceOf[Long]
      val oldRteEntity1: Long = rteData(3).get.asInstanceOf[Long]
      val oldRteEntity2: Long = rteData(4).get.asInstanceOf[Long]
      val validOnDate: Option[Long] = rteData(5).asInstanceOf[Option[Long]]
      val observedDate: Long = rteData(6).get.asInstanceOf[Long]
      deleteRelationToEntity(oldRteRelType, oldRteEntity1, oldRteEntity2)
      val newRTE: RelationToEntity = createRelationToEntity(oldRteRelType, newContainingEntityIdIn, oldRteEntity2, validOnDate, observedDate,
                                                            Some(sortingIndexIn), callerManagesTransactionsIn = true)

      //Something like the next line might have been more efficient than the above code to run, but not to write, given that it adds a complexity about updating
      //the attributesorting table, which might be more tricky in future when something is added to prevent those from being orphaned. The above avoids that or
      //centralizes the question to one place in the code.
      //dbAction("UPDATE RelationToEntity SET (entity_id) = (" + newContainingEntityIdIn + ")" + " where id=" + relationToEntityIdIn)

      commitTrans()
      newRTE
    } catch {
      case e: Exception => throw rollbackWithCatch(e)
    }
  }

  def createGroup(inName: String, allowMixedClassesInGroupIn: Boolean = false): Long = {
    val name: String = escapeQuotesEtc(inName)
    val groupId: Long = getNewKey("RelationToGroupKeySequence")
    dbAction("INSERT INTO grupo (id, name, insertion_date, allow_mixed_classes) " +
             "VALUES (" +
             groupId + ", '" + name + "', " + System.currentTimeMillis() + ", " + (if (allowMixedClassesInGroupIn) "TRUE" else "FALSE") + ")")
    groupId
  }

  /** I.e., make it so the entity has a group in it, which can contain entities.
    * Re dates' meanings: see usage notes elsewhere in code (like inside createTables).
    */
  def createGroupAndRelationToGroup(inEntityId: Long, inRelationTypeId: Long, newGroupNameIn: String, allowMixedClassesInGroupIn: Boolean = false,
                                    inValidOnDate: Option[Long], inObservationDate: Long,
                                    sortingIndexIn: Option[Long], callerManagesTransactionsIn: Boolean = false): (Long, Long) = {
    if (!callerManagesTransactionsIn) beginTrans()
    val groupId: Long = createGroup(newGroupNameIn, allowMixedClassesInGroupIn)
    val (rtgId,_) = createRelationToGroup(inEntityId, inRelationTypeId, groupId, inValidOnDate, inObservationDate, sortingIndexIn, callerManagesTransactionsIn)
    if (!callerManagesTransactionsIn) commitTrans()
    (groupId, rtgId)
  }

  /** I.e., make it so the entity has a relation to a new entity in it.
    * Re dates' meanings: see usage notes elsewhere in code (like inside createTables).
    */
  def createEntityAndRelationToEntity(inEntityId: Long, inRelationTypeId: Long, newEntityNameIn: String, isPublicIn: Option[Boolean],
                                      inValidOnDate: Option[Long], inObservationDate: Long, callerManagesTransactionsIn: Boolean = false): (Long, Long) = {
    val name: String = escapeQuotesEtc(newEntityNameIn)
    if (!callerManagesTransactionsIn) beginTrans()
    val newEntityId: Long = createEntity(name, isPublicIn = isPublicIn)
    val newRte: RelationToEntity = createRelationToEntity(inRelationTypeId, inEntityId, newEntityId, inValidOnDate, inObservationDate, None,
                                                          callerManagesTransactionsIn)
    if (!callerManagesTransactionsIn) commitTrans()
    (newEntityId, newRte.getId)
  }

  /** I.e., make it so the entity has a group in it, which can contain entities.
    * Re dates' meanings: see usage notes elsewhere in code (like inside createTables).
    * @return a tuple containing the id and new sortingIndex: (id, sortingIndex)
    */
  def createRelationToGroup(inEntityId: Long, inRelationTypeId: Long, groupIdIn: Long, inValidOnDate: Option[Long], inObservationDate: Long,
                            sortingIndexIn: Option[Long] = None, callerManagesTransactionsIn: Boolean = false): (Long, Long) = {
    if (!callerManagesTransactionsIn) beginTrans()
    val id: Long = getNewKey("RelationToGroupKeySequence2")
    val sortingIndex = {
      try {
        val sortingIndex: Long = addAttributeSortingRow(inEntityId, PostgreSQLDatabase.getAttributeFormId(Controller.RELATION_TO_GROUP_TYPE), id, sortingIndexIn)
        dbAction("INSERT INTO RelationToGroup (id, entity_id, rel_type_id, group_id, valid_on_date, observation_date) " +
                 "VALUES (" +
                 id + "," + inEntityId + "," + inRelationTypeId + "," + groupIdIn +
                 ", " + (if (inValidOnDate.isEmpty) "NULL" else inValidOnDate.get) + "," + inObservationDate + ")")
        sortingIndex
      }
      catch {
        case e: Exception =>
          if (!callerManagesTransactionsIn) rollbackTrans()
          throw e
      }
    }
    if (!callerManagesTransactionsIn) commitTrans()
    (id, sortingIndex)
  }

  def updateGroup(groupIdIn: Long, inName: String, allowMixedClassesInGroupIn: Boolean = false) {
    val name: String = escapeQuotesEtc(inName)
    dbAction("UPDATE grupo SET (name, allow_mixed_classes)" +
             " = ('" + name + "', " + (if (allowMixedClassesInGroupIn) "TRUE" else "FALSE") +
             ") where id=" + groupIdIn)
  }

  /** Re dates' meanings: see usage notes elsewhere in code (like inside createTables).
    */
  def updateRelationToGroup(entityIdIn: Long, inRelationTypeId: Long, groupIdIn: Long, inValidOnDate: Option[Long], inObservationDate: Long) {
    // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
    // in memory when the db updates, and the behavior gets weird.
    dbAction("UPDATE RelationToGroup SET (valid_on_date, observation_date)" +
             " = (" + (if (inValidOnDate.isEmpty) "NULL" else inValidOnDate.get) + "," + inObservationDate + ")" +
             " where entity_id=" + entityIdIn + " and rel_type_id=" + inRelationTypeId + " and group_id=" + groupIdIn)
  }

  /**
   * @param sortingIndexIn Used because it seems handy (as done in calls to other move methods) to keep it in case one moves many entries: they stay in order.
   * @return the new RelationToGroup's id.
   */
  def moveRelationToGroup(relationToGroupIdIn: Long, newContainingEntityIdIn: Long, sortingIndexIn: Long): Long = {
    beginTrans()
    try {
      val rtgData: Array[Option[Any]] = getAllRelationToGroupDataById(relationToGroupIdIn)
      val oldRtgEntityId: Long = rtgData(2).get.asInstanceOf[Long]
      val oldRtgRelType: Long = rtgData(3).get.asInstanceOf[Long]
      val oldRtgGroupId: Long = rtgData(4).get.asInstanceOf[Long]
      val validOnDate: Option[Long] = rtgData(5).asInstanceOf[Option[Long]]
      val observedDate: Long = rtgData(6).get.asInstanceOf[Long]
      deleteRelationToGroup(oldRtgEntityId, oldRtgRelType, oldRtgGroupId)
      val (newRtgId: Long,_) = createRelationToGroup(newContainingEntityIdIn, oldRtgRelType, oldRtgGroupId, validOnDate, observedDate, Some(sortingIndexIn),
                                                 callerManagesTransactionsIn = true)

      // (see comment at similar commented line in moveRelationToEntity)
      //dbAction("UPDATE RelationToGroup SET (entity_id) = (" + newContainingEntityIdIn + ")" + " where id=" + relationToGroupIdIn)

      commitTrans()
      newRtgId
    } catch {
      case e: Exception => throw rollbackWithCatch(e)
    }
  }

  /** Trying it out with the entity's previous sortingIndex (or whatever is passed in) in case it's more convenient, say, when brainstorming a
    * list then grouping them afterward, to keep them in the same order.  Might be better though just to put them all at the beginning or end; can see....
    */
  def moveEntityFromGroupToGroup(fromGroupIdIn: Long, toGroupIdIn: Long, moveEntityIdIn: Long, sortingIndexIn: Long) {
    beginTrans()
    addEntityToGroup(toGroupIdIn, moveEntityIdIn, Some(sortingIndexIn), callerManagesTransactionsIn = true)
    removeEntityFromGroup(fromGroupIdIn, moveEntityIdIn, callerManagesTransactionsIn = true)
    if (isEntityInGroup(toGroupIdIn, moveEntityIdIn) && !isEntityInGroup(fromGroupIdIn, moveEntityIdIn)) {
      commitTrans()
    } else {
      throw rollbackWithCatch(new OmDatabaseException("Entity didn't get moved properly.  Retry: if predictably reproducible, it should be diagnosed."))
    }
  }

  /** (See comments on moveEntityFromGroupToGroup.)
    */
  def moveEntityFromGroupToEntity(fromGroupIdIn: Long, toEntityIdIn: Long, moveEntityIdIn: Long, sortingIndexIn: Long) {
    beginTrans()
    addHASRelationToEntity(toEntityIdIn, moveEntityIdIn, None, System.currentTimeMillis(), Some(sortingIndexIn))
    removeEntityFromGroup(fromGroupIdIn, moveEntityIdIn, callerManagesTransactionsIn = true)
    commitTrans()
  }

  /** (See comments on moveEntityFromGroupToGroup.)
    */
  def moveEntityFromEntityToGroup(removingRelationToEntityIn: RelationToEntity, targetGroupIdIn: Long, sortingIndexIn: Long) {
    beginTrans()
    addEntityToGroup(targetGroupIdIn, removingRelationToEntityIn.getRelatedId2, Some(sortingIndexIn), callerManagesTransactionsIn = true)
    deleteRelationToEntity(removingRelationToEntityIn.getAttrTypeId, removingRelationToEntityIn.getRelatedId1, removingRelationToEntityIn.getRelatedId2)
    commitTrans()
  }

  val UNUSED_GROUP_ERR1 = "No available index found which is not already used. How would so many be used?"
  val UNUSED_GROUP_ERR2 = "Very unexpected, but could it be that you are running out of available sorting indexes!?" +
                          " Have someone check, before you need to create, for example, a thousand more entities."

  // SEE ALSO METHOD findUnusedAttributeSortingIndex **AND DO MAINTENANCE IN BOTH PLACES**
  // idea: this needs a test, and/or combining with findIdWhichIsNotKeyOfAnyEntity.
  // **ABOUT THE SORTINGINDEX:  SEE the related comment on method addAttributeSortingRow.
  def findUnusedGroupSortingIndex(groupIdIn: Long, startingWithIn: Option[Long] = None): Long = {
    //better idea?  This should be fast because we start in remote regions and return as soon as an unused id is found, probably
    //only one iteration, ever.  (See similar comments elsewhere.)
    @tailrec def findUnusedSortingIndex_helper(gId: Long, workingIndex: Long, counter: Long): Long = {
      if (groupEntrySortingIndexInUse(gId, workingIndex)) {
        if (workingIndex == maxIdValue) {
          // means we did a full loop across all possible ids!?  Doubtful. Probably would turn into a performance problem long before. It's a bug.
          throw new OmDatabaseException(UNUSED_GROUP_ERR1)
        }
        // idea: see comment at similar location in findIdWhichIsNotKeyOfAnyEntity
        if (counter > 1000) throw new OmDatabaseException(UNUSED_GROUP_ERR2)
        findUnusedSortingIndex_helper(gId, workingIndex - 1, counter + 1)
      } else workingIndex
    }

    findUnusedSortingIndex_helper(groupIdIn, startingWithIn.getOrElse(maxIdValue - 1), 0)
  }

  // SEE COMMENTS IN findUnusedGroupSortingIndex **AND DO MAINTENANCE IN BOTH PLACES
  // **ABOUT THE SORTINGINDEX:  SEE the related comment on method addAttributeSortingRow.
  def findUnusedAttributeSortingIndex(entityIdIn: Long, startingWithIn: Option[Long] = None): Long = {
    @tailrec def findUnusedSortingIndex_helper(eId: Long, workingIndex: Long, counter: Long): Long = {
      if (attributeSortingIndexInUse(eId, workingIndex)) {
        if (workingIndex == maxIdValue) {
          throw new OmDatabaseException(UNUSED_GROUP_ERR1)
        }
        if (counter > 1000) throw new OmDatabaseException(UNUSED_GROUP_ERR2)
        findUnusedSortingIndex_helper(eId, workingIndex - 1, counter + 1)
      } else workingIndex
    }
    findUnusedSortingIndex_helper(entityIdIn, startingWithIn.getOrElse(maxIdValue - 1), 0)
  }

  /** I.e., insert an entity into a group of entities. Using a default value for the sorting_index because user can set it if/as desired;
    * the max (ie putting it at the end) might be the least often surprising if the user wonders where one went....
    * **ABOUT THE SORTINGINDEX*:  SEE the related comment on method addAttributeSortingRow.
    */
  def addEntityToGroup(groupIdIn: Long, containedEntityIdIn: Long, sortingIndexIn: Option[Long] = None, callerManagesTransactionsIn: Boolean = false) {
    // IF THIS CHANGES ALSO DO MAINTENANCE IN SIMILAR METHOD addAttributeSortingRow
    if (!callerManagesTransactionsIn) beginTrans()

    // start from the beginning index, if it's the 1st record (otherwise later sorting/renumbering gets messed up if we start w/ the last #):
    val sortingIndex = {
      val index = if (sortingIndexIn.isDefined) sortingIndexIn.get
      // start with an increment off the min or max, so that later there is room to sort something before or after it, manually:
      else if (getGroupSize(groupIdIn) == 0) minIdValue + 9999
      else maxIdValue - 9999

      if (groupEntrySortingIndexInUse(groupIdIn, index))
        findUnusedGroupSortingIndex(groupIdIn)
      else
        index
    }

    dbAction("insert into EntitiesInAGroup (group_id, entity_id, sorting_index) values (" + groupIdIn + "," + containedEntityIdIn + "," +
             "" + sortingIndex + ")")
    // idea: do this check sooner in this method?:
    val mixedClassesAllowed: Boolean = areMixedClassesAllowed(groupIdIn)
    if ((!mixedClassesAllowed) && hasMixedClasses(groupIdIn)) {
      if (!callerManagesTransactionsIn) rollbackTrans()
      throw new OmDatabaseException(PostgreSQLDatabase.MIXED_CLASSES_EXCEPTION)
    }
    if (!callerManagesTransactionsIn) commitTrans()
  }

  /**
   * @param sortingIndexIn is currently passed by callers with a default guess, not a guaranteed good value, so if it is in use, this ~tries to find a good one.
   *                       An alternate approach could be to pass in a callback to some controller (menu) code, which this can call if it thinks it
   *                       is taking a long time to find a free value, to give the eventual caller chance to give up if needed.  Or just pass in a known
   *                       good value or call the renumberSortingIndexes method.
   * @return the sorting_index value that is actually used.
   */
  def addAttributeSortingRow(entityIdIn: Long, attributeFormIdIn: Long, attributeIdIn: Long, sortingIndexIn: Option[Long] = None): Long = {
    // SEE COMMENTS IN SIMILAR METHOD: addEntityToGroup.  **AND DO MAINTENANCE. IN BOTH PLACES.
    // Should probably be called from inside a transaction (which isn't managed in this method, since all its current callers do it.)
    val sortingIndex = {
      val index = {
        if (sortingIndexIn.isDefined) sortingIndexIn.get
        // start with an increment off the min or max, so that later there is room to sort something before or after it, manually:
        else if (getAttrCount(entityIdIn) == 0) minIdValue + 9999
        else maxIdValue - 9999
      }
      if (attributeSortingIndexInUse(entityIdIn, index))
        findUnusedAttributeSortingIndex(entityIdIn)
      else
        index
    }
    dbAction("insert into AttributeSorting (entity_id, attribute_form_id, attribute_id, sorting_index) " +
             "values (" + entityIdIn + "," + attributeFormIdIn + "," + attributeIdIn + "," + sortingIndex + ")")
    sortingIndex
  }

  def areMixedClassesAllowed(groupId: Long): Boolean = {
    val rows = dbQuery("select allow_mixed_classes from grupo where id =" + groupId, "Boolean")
    val mixedClassesAllowed: Boolean = rows.head(0).get.asInstanceOf[Boolean]
    mixedClassesAllowed
  }

  def hasMixedClasses(inGroupId: Long): Boolean = {
    // Enforce that all entities in so-marked groups have the same class (or they all have no class; too bad).
    // (This could be removed or modified, but some user scripts attached to groups might (someday?) rely on their uniformity, so this
    // and the fact that you can have a group all of which don't have any class, is experimental.  This is optional, per
    // group.  I.e., trying it that way now to see whether it removes desired flexibility
    // at a cost higher than the benefit of uniformity for later user code operating on groups.  This might be better in a constraint,
    // but after trying for a while I hadn't made the syntax work right.

    // (Had to ask for them all and expect 1, instead of doing a count, because for some reason "select count(class_id) ... group by class_id" doesn't
    // group, and you get > 1 when I wanted just 1. This way it seems to work if I just check the # of rows returned.)
    val numClassesInGroupsEntities = dbQuery("select class_id from EntitiesInAGroup eiag, entity e" +
                                             " where eiag.entity_id=e.id and group_id=" + inGroupId +
                                             " and class_id is not null" +
                                             " group by class_id",
                                             "Long").size
    // nulls don't show up in a count(class_id), so get those separately
    val numNullClassesInGroupsEntities = extractRowCountFromCountQuery("select count(entity_id) from EntitiesInAGroup eiag, entity e" +
                                                                       " where eiag.entity_id=e.id" + " and group_id=" + inGroupId +
                                                                       " and class_id is NULL ")
    if (numClassesInGroupsEntities > 1 ||
        (numClassesInGroupsEntities >= 1 && numNullClassesInGroupsEntities > 0)) {
      true
    } else false
  }

  def createEntity(inName: String, inClassId: Option[Long] = None, isPublicIn: Option[Boolean] = None): /*id*/ Long = {
    val name: String = escapeQuotesEtc(inName)
    if (name == null || name.length == 0) throw new OmDatabaseException("Name must have a value.")
    val id: Long = getNewKey("EntityKeySequence")
    val sql: String = "INSERT INTO Entity (id, insertion_date, name, public" + (if (inClassId.isDefined) ", class_id" else "") + ")" +
                      " VALUES (" + id + "," + System.currentTimeMillis() + ",'" + name + "'," +
                      (if (isPublicIn.isEmpty) "NULL" else isPublicIn.get) +
                      (if (inClassId.isDefined) "," + inClassId.get else "") + ")"
    dbAction(sql)
    id
  }

  def createRelationType(inName: String, inNameInReverseDirection: String, inDirectionality: String): /*id*/ Long = {
    val nameInReverseDirection: String = escapeQuotesEtc(inNameInReverseDirection)
    val name: String = escapeQuotesEtc(inName)
    val directionality: String = escapeQuotesEtc(inDirectionality)
    if (name == null || name.length == 0) throw new OmDatabaseException("Name must have a value.")
    beginTrans()
    try {
      val id: Long = getNewKey("EntityKeySequence")
      dbAction("INSERT INTO Entity (id, insertion_date, name) VALUES (" + id + "," + System.currentTimeMillis() + ",'" + name + "')")
      dbAction("INSERT INTO RelationType (entity_id, name_in_reverse_direction, directionality) VALUES (" + id + ",'" + nameInReverseDirection + "'," +
               "'" + directionality + "')")
      commitTrans()
      id
    } catch {
      case e: Exception => throw rollbackWithCatch(e)
    }
  }

  def rollbackWithCatch(t: Throwable): Throwable = {
    var rollbackException: Option[Throwable] = None
    try {
      rollbackTrans()
    } catch {
      case e: Exception =>
        rollbackException = Some(e)
    }
    if (rollbackException.isEmpty) t
    else {
      rollbackException.get.addSuppressed(t)
      val exc = new OmDatabaseException("See the chained messages for ALL: the cause of rollback failure, AND for the original failure(s).",
                                        rollbackException.get)
      exc
    }
  }

  def deleteEntity(inId: Long, callerManagesTransactionsIn: Boolean = false) = {
    // idea: (also on task list i think but) we should not delete entities until dealing with their use as attrtypeids etc!
    if (!callerManagesTransactionsIn) beginTrans()
    deleteObjects("EntitiesInAGroup", "where entity_id=" + inId, -1, callerManagesTransactions = true)
    deleteObjects(Controller.ENTITY_TYPE, "where id=" + inId, 1, callerManagesTransactions = true)
    deleteObjects("AttributeSorting", "where entity_id=" + inId, -1, callerManagesTransactions = true)
    if (!callerManagesTransactionsIn) commitTrans()
  }

  def archiveEntity(inId: Long, callerManagesTransactionsIn: Boolean = false) = {
    archiveObjects(Controller.ENTITY_TYPE, "where id=" + inId, 1, callerManagesTransactionsIn)
  }

  def unarchiveEntity(inId: Long, callerManagesTransactionsIn: Boolean = false) = {
    archiveObjects(Controller.ENTITY_TYPE, "where id=" + inId, 1, callerManagesTransactionsIn, unarchive = true)
  }

  def deleteQuantityAttribute(inID: Long) = deleteObjectById(Controller.QUANTITY_TYPE, inID)

  def deleteTextAttribute(inID: Long) = deleteObjectById(Controller.TEXT_TYPE, inID)

  def deleteDateAttribute(inID: Long) = deleteObjectById(Controller.DATE_TYPE, inID)

  def deleteBooleanAttribute(inID: Long) = deleteObjectById(Controller.BOOLEAN_TYPE, inID)

  def deleteFileAttribute(inID: Long) = deleteObjectById(Controller.FILE_TYPE, inID)

  def deleteRelationToEntity(inRelTypeId: Long, inEntityId1: Long, inEntityId2: Long) {
    deleteObjects(Controller.RELATION_TO_ENTITY_TYPE, "where rel_type_id=" + inRelTypeId + " and entity_id=" + inEntityId1 + " and entity_id_2=" + inEntityId2)
  }

  def deleteRelationToGroup(entityIdIn: Long, relTypeIdIn: Long, groupIdIn: Long) {
    deleteObjects(Controller.RELATION_TO_GROUP_TYPE, "where entity_id=" + entityIdIn + " and rel_type_id=" + relTypeIdIn + " and group_id=" + groupIdIn)
  }

  def deleteGroupAndRelationsToIt(inId: Long) {
    beginTrans()
    try {
      val entityCount: Long = getGroupSize(inId)
      deleteObjects("EntitiesInAGroup", "where group_id=" + inId, entityCount, callerManagesTransactions = true)
      val numGroups = getRelationToGroupCountByGroup(inId)
      deleteObjects(Controller.RELATION_TO_GROUP_TYPE, "where group_id=" + inId, numGroups, callerManagesTransactions = true)
      deleteObjects("grupo", "where id=" + inId, 1, callerManagesTransactions = true)
    }
    catch {
      case e: Exception => throw rollbackWithCatch(e)
    }
    commitTrans()
  }

  def removeEntityFromGroup(groupIdIn: Long, inContainedEntityId: Long, callerManagesTransactionsIn: Boolean = false) {
    deleteObjects("EntitiesInAGroup", "where group_id=" + groupIdIn + " and entity_id=" + inContainedEntityId,
                  callerManagesTransactions = callerManagesTransactionsIn)
  }

  /** I hope you have a backup. */
  def deleteGroupRelationsToItAndItsEntries(groupidIn: Long) {
    beginTrans()
    try {
      val entityCount = getGroupSize(groupidIn)

      def deleteRelationToGroupAndALL_recursively(inGroupId: Long): (Long, Long) = {
        val entityIds: List[Array[Option[Any]]] = dbQuery("select entity_id from entitiesinagroup where group_id=" + inGroupId, "Long")
        val deletions1 = deleteObjects("entitiesinagroup", "where group_id=" + inGroupId, entityCount, callerManagesTransactions = true)
        // Have to delete these 2nd because of a constraint on EntitiesInAGroup:
        // idea: is there a temp table somewhere that these could go into instead, for efficiency?
        // idea: batch these, would be much better performance.
        // idea: BUT: what is the length limit: should we do it it sets of N to not exceed sql command size limit?
        // idea: (also on task list i think but) we should not delete entities until dealing with their use as attrtypeids etc!
        for (id <- entityIds) {
          deleteObjects(Controller.ENTITY_TYPE, "where id=" + id(0).get.asInstanceOf[Long], 1, callerManagesTransactions = true)
        }

        val deletions2 = 0
        //and finally:
        // (passing -1 for rows expected, because there either could be some, or none if the group is not contained in any entity.)
        deleteObjects(Controller.RELATION_TO_GROUP_TYPE, "where group_id=" + inGroupId, -1, callerManagesTransactions = true)
        deleteObjects("grupo", "where id=" + inGroupId, 1, callerManagesTransactions = true)
        (deletions1, deletions2)
      }
      val (deletions1, deletions2) = deleteRelationToGroupAndALL_recursively(groupidIn)
      require(deletions1 + deletions2 == entityCount)
    }
    catch {
      case e: Exception => throw rollbackWithCatch(e)
    }
    commitTrans()
  }

  def deleteRelationType(inID: Long) {
    // One possibility is that this should ALWAYS fail because it is done by deleting the entity, which cascades.
    // but that's more confusing to the programmer using the database layer's api calls, because they
    // have to know to delete an Entity instead of a RelationType. So we just do the desired thing here
    // instead, and the delete cascades.
    // Maybe those tables should be separated so this is its own thing? for performance/clarity?
    // like *attribute and relation don't have a parent 'attribute' table?  But see comments
    // in createTables where this one is created.
    deleteObjects(Controller.ENTITY_TYPE, "where id=" + inID)
  }

  def getSystemEntityId: Long = {
    val ids: Option[List[Long]] = findEntityOnlyIdsByName(PostgreSQLDatabase.systemEntityName)
    require(ids.get.size == 1)
    ids.get.head
  }

  /** Creates the preference if it doesn't already exist.  */
  def setUserPreference_Boolean(nameIn: String, valueIn: Boolean) = {
    val preferencesContainerId: Long = getPreferencesContainerId
    val result = getUserPreference2(preferencesContainerId, nameIn, PostgreSQLDatabase.PREF_TYPE_BOOLEAN)
    val preferenceInfo: Option[(Long, Boolean)] = result.asInstanceOf[Option[(Long,Boolean)]]
    if (preferenceInfo.isDefined) {
      val preferenceAttributeId: Long = preferenceInfo.get._1
      val attribute = new BooleanAttribute(this, preferenceAttributeId)
      updateBooleanAttribute(attribute.getId, attribute.getParentId, attribute.getAttrTypeId, valueIn, attribute.getValidOnDate, attribute.getObservationDate)
    } else {
      val HASrelationTypeId = findRelationType(PostgreSQLDatabase.theHASrelationTypeName, Some(1))(0)
      val preferenceEntityId: Long = createEntityAndRelationToEntity(preferencesContainerId, HASrelationTypeId, nameIn, None,
                                                                     Some(System.currentTimeMillis()), System.currentTimeMillis())._1
      // (For about the attr_type_id value (2nd parm), see comment about that field, in method getUserPreference_Boolean2 below.)
      createBooleanAttribute(preferenceEntityId, preferenceEntityId, valueIn, Some(System.currentTimeMillis()), System.currentTimeMillis())
    }
  }

  def getUserPreference_Boolean(preferenceNameIn: String, defaultValueIn: Option[Boolean] = None): Option[Boolean] = {
    val pref = getUserPreference2(getPreferencesContainerId, preferenceNameIn, PostgreSQLDatabase.PREF_TYPE_BOOLEAN)
    if (pref.isEmpty) {
      defaultValueIn
    } else {
      Some(pref.get.asInstanceOf[(Long,Boolean)]._2)
    }
  }

  /** Creates the preference if it doesn't already exist.  */
  def setUserPreference_EntityId(nameIn: String, entityIdIn: Long) = {
    val preferencesContainerId: Long = getPreferencesContainerId
    val result = getUserPreference2(preferencesContainerId, nameIn, PostgreSQLDatabase.PREF_TYPE_ENTITY_ID)
    val preferenceInfo: Option[(Long, Long, Long)] = result.asInstanceOf[Option[(Long,Long,Long)]]
    if (preferenceInfo.isDefined) {
      val relationTypeId: Long = preferenceInfo.get._1
      val entityId1: Long = preferenceInfo.get._2
      val entityId2: Long = preferenceInfo.get._3
      // didn't bother to put these 2 calls in a transaction because this is likely to be so rarely used and easily fixed by user if it fails (from default
      // entity setting on any entity menu)
      deleteRelationToEntity(relationTypeId, entityId1, entityId2)
      // (Using entityId1 instead of (the likely identical) preferencesContainerId, in case this RTE was originally found down among some
      // nested preferences (organized for user convenience) under here, in order to keep that organization.)
      createRelationToEntity(relationTypeId, entityId1, entityIdIn, Some(System.currentTimeMillis()), System.currentTimeMillis())
    } else {
      val HASrelationTypeId = findRelationType(PostgreSQLDatabase.theHASrelationTypeName, Some(1))(0)
      val preferenceEntityId: Long = createEntityAndRelationToEntity(preferencesContainerId, HASrelationTypeId, nameIn, None,
                                                                     Some(System.currentTimeMillis()), System.currentTimeMillis())._1
      createRelationToEntity(HASrelationTypeId, preferenceEntityId, entityIdIn, Some(System.currentTimeMillis()), System.currentTimeMillis())
    }
  }

  def getUserPreference_EntityId(preferenceNameIn: String, defaultValueIn: Option[Long] = None): Option[Long] = {
    val pref = getUserPreference2(getPreferencesContainerId, preferenceNameIn, PostgreSQLDatabase.PREF_TYPE_ENTITY_ID)
    if (pref.isEmpty) {
      defaultValueIn
    } else {
      Some(pref.get.asInstanceOf[(Long,Long,Long)]._3)
    }
  }

  def getUserPreference2(preferencesContainerIdIn: Long, preferenceNameIn: String, preferenceType: String): Option[Any] = {
    // (Passing a smaller numeric parameter to findContainedEntityIds for levelsRemainingIn, so that in the (very rare) case where one does not
    // have a default entity set at the *top* level of the preferences under the system entity, and there are links there to entities with many links
    // to others, then it still won't take too long to traverse them all at startup when searching for the default entity.  But still allowing for
    // preferences to be nested up to that many levels (3 as of this writing).
    val foundPreferences: mutable.TreeSet[Long] = findContainedEntityIds(new mutable.TreeSet[Long], preferencesContainerIdIn, preferenceNameIn, 3)
    if (foundPreferences.isEmpty) {
      None
    } else {
      require(foundPreferences.size == 1, "Under the entity \"" + getEntityName(preferencesContainerIdIn) + "\" (" + preferencesContainerIdIn +
                                          ", possibly under " + PostgreSQLDatabase.systemEntityName +
                                          "), there is (eventually) more than one entity with the name \"" + preferenceNameIn +
                                          "\", so the program does not know which one to use for this.")
      val preferenceEntity = new Entity(this, foundPreferences.firstKey)
      val relevantAttributeRows: List[Array[Option[Any]]] = {
        if (preferenceType == PostgreSQLDatabase.PREF_TYPE_BOOLEAN) {
          // (Using the preferenceEntity.getId for attr_type_id, just for convenience since it seemed as good as any.  ALSO USED IN THE SAME WAY,
          // IN setUserPreference METHOD CALL TO createBooleanAttribute!)
          val sql2 = "select id, booleanValue from booleanattribute where entity_id=" + preferenceEntity.getId + " and attr_type_id=" + preferenceEntity.getId
          dbQuery(sql2, "Long,Boolean")
        } else if (preferenceType == PostgreSQLDatabase.PREF_TYPE_ENTITY_ID) {
          val sql2 = "select rel_type_id, entity_id, entity_id_2 from relationtoentity where entity_id=" + preferenceEntity.getId
          dbQuery(sql2, "Long,Long,Long")
        } else {
          throw new OmDatabaseException("Unexpected preferenceType: " + preferenceType)
        }
      }
      if (relevantAttributeRows.isEmpty) {
        // at this point we probably have a preference entity but not the expected attribute inside it that holds the actual useful information, so the
        // user needs to go delete the bad preference entity or re-create the attribute.
        // Idea: should there be a good way to *tell* them that, from here?
        // Or, just delete the bad preference (self-cleanup). If it was the public/private display toggle, its absence will cause errors (though it is a
        // very unlikely situation here), and it will be fixed on restarting the app (or starting another instance), via the createExpectedData method.
        deleteEntity(preferenceEntity.getId)
        None
      } else {
        require(relevantAttributeRows.size == 1, "Under the entity " + getEntityName(preferenceEntity.getId) + " (" + preferenceEntity.getId +
                                                     "), there are " + relevantAttributeRows.size +
                                                 (if (preferenceType == PostgreSQLDatabase.PREF_TYPE_BOOLEAN) {
                                                   " BooleanAttributes with the relevant type (" + preferenceNameIn + "," + preferencesContainerIdIn + "), "
                                                  } else if (preferenceType == PostgreSQLDatabase.PREF_TYPE_ENTITY_ID) {
                                                     " RelationToEntity values "
                                                  } else {
                                                     throw new OmDatabaseException("Unexpected preferenceType: " + preferenceType)
                                                  }
                                                 ) +
                                                 "so the program does not know what to use for this.  There should be *one*.")
        if (preferenceType == PostgreSQLDatabase.PREF_TYPE_BOOLEAN) {
          val preferenceId: Long = relevantAttributeRows.head(0).get.asInstanceOf[Long]
          val preferenceValue: Boolean = relevantAttributeRows.head(1).get.asInstanceOf[Boolean]
          Some((preferenceId, preferenceValue))
        } else if (preferenceType == PostgreSQLDatabase.PREF_TYPE_ENTITY_ID) {
          val relTypeId: Long = relevantAttributeRows.head(0).get.asInstanceOf[Long]
          val entityId1: Long = relevantAttributeRows.head(1).get.asInstanceOf[Long]
          val entityId2: Long = relevantAttributeRows.head(2).get.asInstanceOf[Long]
          Some((relTypeId, entityId1, entityId2))
        } else {
          throw new OmDatabaseException("Unexpected preferenceType: " + preferenceType)
        }
      }
    }
  }

  def getRelationToEntityByName(containingEntityIdIn: Long, nameIn: String): Option[Long] = {
    val sql = "select rte.entity_id_2 from relationtoentity rte, entity e where rte.entity_id=" + containingEntityIdIn +
              (if (!showAllArchivedEntities) {
                " and (not e.archived)"
              } else {
                ""
              }) +
              " and rte.entity_id_2=e.id and e.name='" + nameIn + "'"
    val relatedEntityIdRows = dbQuery(sql, "Long")
    if (relatedEntityIdRows.isEmpty) {
      None
    } else {
      require(relatedEntityIdRows.size == 1, "Under the entity " + getEntityName(containingEntityIdIn) + "(" + containingEntityIdIn +
                                             "), there is more one than entity with the name \"" + Controller.USER_PREFERENCES +
                                             "\", so the program does not know which one to use for this.")
      Some(relatedEntityIdRows.head(0).get.asInstanceOf[Long])
    }
  }

  /** This should never return None, except when method createExpectedData is called for the first time in a given database. */
  def getPreferencesContainerId: Long = {
    val relatedEntityId = getRelationToEntityByName(getSystemEntityId, Controller.USER_PREFERENCES)
    if (relatedEntityId.isEmpty) {
      throw new OmDatabaseException("This should never happen: method createExpectedData should be run at startup to create this part of the data.")
    }
    relatedEntityId.get
  }

  def getEntityCount: Long = extractRowCountFromCountQuery("SELECT count(1) from Entity " +
                                                           (if (!showAllArchivedEntities) {
                                                             "where (not archived)"
                                                           } else {
                                                             ""
                                                           })
                                                          )

  def getClassCount(inEntityId: Option[Long] = None): Long = {
    val whereClause = if (inEntityId.isDefined) " where defining_entity_id=" + inEntityId.get else ""
    extractRowCountFromCountQuery("SELECT count(1) from class" + whereClause)
  }

  def getGroupSortingIndex(groupIdIn: Long, entityIdIn: Long): Long = {
    val row = dbQueryWrapperForOneRow("select sorting_index from EntitiesInAGroup where group_id=" + groupIdIn + " and entity_id=" + entityIdIn, "Long")
    row(0).get.asInstanceOf[Long]
  }

  def getEntityAttributeSortingIndex(entityIdIn: Long, attributeFormIdIn: Long, attributeIdIn: Long): Long = {
    val row = dbQueryWrapperForOneRow("select sorting_index from AttributeSorting where entity_id=" + entityIdIn + " and attribute_form_id=" +
                                      attributeFormIdIn + " and attribute_id=" + attributeIdIn, "Long")
    row(0).get.asInstanceOf[Long]
  }

  def getHighestSortingIndex(groupIdIn: Long): Long = {
    val rows: List[Array[Option[Any]]] = dbQuery("select max(sorting_index) from EntitiesInAGroup where group_id=" + groupIdIn, "Long")
    require(rows.size == 1)
    rows.head(0).get.asInstanceOf[Long]
  }

  def renumberSortingIndexes(entityIdOrGroupIdIn: Long, callerManagesTransactionsIn: Boolean = false, isEntityAttrsNotGroupEntries: Boolean = true) {
    //This used to be called "renumberAttributeSortingIndexes" before it was merged with "renumberGroupSortingIndexes" (very similar).
    val numberOfEntries: Long = {
      if (isEntityAttrsNotGroupEntries) getAttrCount(entityIdOrGroupIdIn, includeArchivedEntitiesIn = true)
      else getGroupSize(entityIdOrGroupIdIn)
    }
    if (numberOfEntries != 0) {
      // (like a number line so + 1, then add 1 more (so + 2) in case we use up some room on the line due to "attributeSortingIndexInUse" (below))
      val numberOfSegments = numberOfEntries + 2
      // ( * 2 on next line, because the minIdValue is negative so there is a larger range to split up, but
      // doing so without exceeding the value of a Long during the calculation.)
      val increment: Long = (maxIdValue.asInstanceOf[Float] / numberOfSegments * 2).asInstanceOf[Long]
      // (start with an increment so that later there is room to sort something prior to it, manually)
      var next: Long = minIdValue + increment
      var previous: Long = minIdValue
      if (!callerManagesTransactionsIn) beginTrans()
      try {
        val data: List[Array[Option[Any]]] = {
          if (isEntityAttrsNotGroupEntries) getEntityAttributeSortingData(entityIdOrGroupIdIn)
          else getGroupEntriesData(entityIdOrGroupIdIn)
        }
        require(data.size == numberOfEntries, "Unexpected data state: data.size=" + data.size + " and numberOfEntries=" + numberOfEntries  + ".")
        for (entry <- data) {
          if (isEntityAttrsNotGroupEntries) {
            while (attributeSortingIndexInUse(entityIdOrGroupIdIn, next)) {
              // Renumbering might choose already-used numbers, because it always uses the same algorithm.  This causes a constraint violation (unique index)
              // , so
              // get around that with a (hopefully quick & simple) increment to get the next unused one.  If they're all used...that's a surprise.
              // Idea: also fix this bug in the case where it's near the end & the last #s are used: wrap around? when give err after too many loops: count?
              next += 1
            }
          } else {
            while (groupEntrySortingIndexInUse(entityIdOrGroupIdIn, next)) {
              next += 1
            }
          }
          // (make sure a bug didn't cause wraparound w/in the set of possible Long values)
          require(previous < next && next < maxIdValue, "Requirement failed for values previous, next, and maxIdValue: " + previous + ", " + next + ", " +
                                                        maxIdValue)
          if (isEntityAttrsNotGroupEntries) {
            val formId: Long = entry(0).get.asInstanceOf[Int]
            val attributeId: Long = entry(1).get.asInstanceOf[Long]
            updateAttributeSorting(entityIdOrGroupIdIn, formId, attributeId, next)
          } else {
            val id: Long = entry(0).get.asInstanceOf[Long]
            updateEntityInAGroup(entityIdOrGroupIdIn, id, next)
          }
          previous = next
          next += increment
        }
      }
      catch {
        case e: Exception =>
          if (!callerManagesTransactionsIn) rollbackTrans()
          throw e
      }

      // require: just to confirm that the generally expected behavior happened, not a requirement other than that:
      // (didn't happen in case of newly added entries w/ default values....
      // idea: could investigate further...does it matter or imply anything for adding entries to *brand*-newly created groups? Is it related
      // to the fact that when doing that, the 2nd entry goes above, not below the 1st, and to move it down you have to choose the down 1 option
      // *twice* for some reason (sometimes??)? And to the fact that deleting an entry selects the one above, not below, for next highlighting?)
      // (See also a comment somewhere else 4 poss. issue that refers, related, to this method name.)
      //require((maxIDValue - next) < (increment * 2))

      if (!callerManagesTransactionsIn) commitTrans()
    }
  }

  def classLimit(limitByClass: Boolean, inClassId: Option[Long]): String = {
    if (limitByClass) {
      if (inClassId.isDefined) {
        " and e.class_id=" + inClassId.get + " "
      } else {
        " and e.class_id is NULL "
      }
    } else ""
  }

  /** Excludes those entities that are really relationtypes, attribute types, or quantity units.
    *
    * The parameter limitByClass decides whether any limiting is done at all: if true, the query is
    * limited to entities having the class specified by inClassId (even if that is None).
    *
    * The parameter templateEntity *further* limits, if limitByClass is true, by omitting the templateEntity from the results (e.g., to help avoid
    * counting that one when deciding whether it is OK to delete the class).
    * */
  def getEntitiesOnlyCount(inClassId: Option[Long] = None, limitByClass: Boolean = false,
                           templateEntity: Option[Long] = None): Long = {
    extractRowCountFromCountQuery("SELECT count(1) from Entity e where " +
                                  (if (!showAllArchivedEntities) {
                                    "(not archived) and "
                                  } else {
                                    ""
                                  }) +
                                  "true " +
                                  classLimit(limitByClass, inClassId) +
                                  (if (limitByClass && templateEntity.isDefined) " and id != " + templateEntity.get else "") +
                                  " and id in " +
                                  "(select id from entity " + limitToEntitiesOnly(ENTITY_ONLY_SELECT_PART) +
                                  ")")
  }

  def getRelationTypeCount: Long = extractRowCountFromCountQuery("select count(1) from RelationType")

  def getAttrCount(entityIdIn: Long, includeArchivedEntitiesIn: Boolean = false): Long = {
    getQuantityAttributeCount(entityIdIn) +
    getTextAttributeCount(entityIdIn) +
    getDateAttributeCount(entityIdIn) +
    getBooleanAttributeCount(entityIdIn) +
    getFileAttributeCount(entityIdIn) +
    getRelationToEntityCount(entityIdIn, includeArchivedEntitiesIn) +
    getRelationToGroupCountByEntity(Some(entityIdIn))
  }

  def getAttributeSortingRowsCount(entityIdIn: Option[Long] = None): Long = {
    val sql = "select count(1) from AttributeSorting " + (if (entityIdIn.isDefined) "where entity_id=" + entityIdIn.get else "")
    extractRowCountFromCountQuery(sql)
  }

  def getQuantityAttributeCount(inEntityId: Long): Long = {
    extractRowCountFromCountQuery("select count(1) from QuantityAttribute where entity_id=" + inEntityId)
  }

  def getTextAttributeCount(inEntityId: Long): Long = {
    extractRowCountFromCountQuery("select count(1) from TextAttribute where entity_id=" + inEntityId)
  }

  def getDateAttributeCount(inEntityId: Long): Long = {
    extractRowCountFromCountQuery("select count(1) from DateAttribute where entity_id=" + inEntityId)
  }

  def getBooleanAttributeCount(inEntityId: Long): Long = {
    extractRowCountFromCountQuery("select count(1) from BooleanAttribute where entity_id=" + inEntityId)
  }

  def getFileAttributeCount(inEntityId: Long): Long = {
    extractRowCountFromCountQuery("select count(1) from FileAttribute where entity_id=" + inEntityId)
  }

  def getRelationToEntityCount(inEntityId: Long, includeArchivedEntities: Boolean = true): Long = {
    var sql = "select count(1) from entity eContaining, RelationToEntity rte, entity eContained " +
              " where eContaining.id=rte.entity_id and rte.entity_id=" + inEntityId +
              " and rte.entity_id_2=eContained.id"
    if (!includeArchivedEntities && !showAllArchivedEntities) sql += " and (not eContained.archived)"
    extractRowCountFromCountQuery(sql)
  }

  /** if 1st parm is None, gets all. */
  def getRelationToGroupCountByEntity(inEntityId: Option[Long]): Long = {
    extractRowCountFromCountQuery("select count(1) from relationtogroup" + (if (inEntityId.isEmpty) "" else " where entity_id=" + inEntityId.get))
  }

  def getRelationToGroupCountByGroup(inGroupId: Long): Long = {
    extractRowCountFromCountQuery("select count(1) from relationtogroup where group_id=" + inGroupId)
  }

  def getRelationToGroupsByGroup(groupIdIn: Long, startingIndexIn: Long, maxValsIn: Option[Long] = None): java.util.ArrayList[RelationToGroup] = {
    val sql: String = "select rtg.id, rtg.entity_id, rtg.rel_type_id, rtg.group_id, rtg.valid_on_date, rtg.observation_date, asort.sorting_index" +
                      " from RelationToGroup rtg, AttributeSorting asort where group_id=" + groupIdIn +
                      " and rtg.entity_id=asort.entity_id and asort.attribute_form_id=" + PostgreSQLDatabase.getAttributeFormId(Controller.RELATION_TO_GROUP_TYPE) +
                      " and rtg.id=asort.attribute_id"
    val earlyResults = dbQuery(sql, "Long,Long,Long,Long,Long,Long,Long")
    val finalResults = new java.util.ArrayList[RelationToGroup]
    // idea: should the remainder of this method be moved to RelationToGroup, so the persistence layer doesn't know anything about the Model? (helps avoid
    // circular dependencies? is a cleaner design?)
    for (result <- earlyResults) {
      // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
      //finalResults.add(result(0).get.asInstanceOf[Long], new Entity(this, result(1).get.asInstanceOf[Long]))
      val rtg: RelationToGroup = new RelationToGroup(this, result(0).get.asInstanceOf[Long], result(1).get.asInstanceOf[Long],
                                                     result(2).get.asInstanceOf[Long], result(3).get.asInstanceOf[Long],
                                                     if (result(4).isEmpty) None else Some(result(4).get.asInstanceOf[Long]), result(5).get.asInstanceOf[Long],
                                                     result(6).get.asInstanceOf[Long])
      finalResults.add(rtg)
    }
    require(finalResults.size == earlyResults.size)
    finalResults
  }

  def getGroupCount: Long = {
    extractRowCountFromCountQuery("select count(1) from grupo")
  }

  /**
   * @param groupIdIn groupId
   * @param includeArchivedEntities true/false means select only archived/non-archived entities; None there means BOTH archived and non-archived (all).
   */
  def getGroupSize(groupIdIn: Long, includeArchivedEntities: Option[Boolean] = None): Long = {
    val archivedSqlCondition: String = if (includeArchivedEntities.isEmpty) "true"
    else if (includeArchivedEntities.get) "archived"
    else "(not archived)"
    extractRowCountFromCountQuery("select count(1) from entity e, EntitiesInAGroup eiag where e.id=eiag.entity_id and " + archivedSqlCondition + " and eiag" +
                                  ".group_id=" + groupIdIn)
  }

  /** For all groups to which the parameter belongs, returns a collection of the *containing* RelationToGroups, in the form of "entityName -> groupName"'s.
    * This is useful for example when one is about
    * to delete an entity and we want to warn first, showing where it is contained.
    */
  def getContainingRelationToGroupDescriptions(inEntityId: Long, inLimit: Option[Long] = None): Array[String] = {
    val rows: List[Array[Option[Any]]] = dbQuery("select e.name, grp.name, grp.id from entity e, relationtogroup rtg, " +
                                                 "grupo grp where " +
                                                 (if (!showAllArchivedEntities) {
                                                   "(not archived) and "
                                                 } else {
                                                   ""
                                                 }) +
                                                 "e.id = rtg.entity_id" +
                                                 " and rtg.group_id = grp.id and rtg.group_id in (SELECT group_id from entitiesinagroup where entity_id=" +
                                                 inEntityId + ")" +
                                                 " order by grp.id limit " + checkIfShouldBeAllResults(inLimit), "String,String,Long")
    var results: List[String] = Nil
    for (row <- rows) {
      val entityName = row(0).get.asInstanceOf[String]
      val groupName = row(1).get.asInstanceOf[String]
      results = entityName + "->" + groupName :: results
    }

    results.reverse.toArray
  }

  /** For a given group, find all the RelationToGroup's that contain entities that contain the provided group id, and return their groupIds.
    * What is really the best name for this method (concise but clear on what it does)?
    */
  def getGroupsContainingEntitysGroupsIds(groupIdIn: Long, inLimit: Option[Long] = Some(5)): List[Array[Option[Any]]] = {
    //get every entity that contains a rtg that contains this group:
    val containingEntityIdList: List[Array[Option[Any]]] = dbQuery("SELECT entity_id from relationtogroup where group_id=" + groupIdIn +
                                                                   " order by entity_id limit " + checkIfShouldBeAllResults(inLimit), "Long")
    var containingEntityIds: String = ""
    //for all those entity ids, get every rtg id containing that entity
    for (row <- containingEntityIdList) {
      val entityId: Long = row(0).get.asInstanceOf[Long]
      containingEntityIds += entityId
      containingEntityIds += ","
    }
    if (containingEntityIds.nonEmpty) {
      // remove the last comma
      containingEntityIds = containingEntityIds.substring(0, containingEntityIds.length - 1)
      val rtgRows: List[Array[Option[Any]]] = dbQuery("SELECT group_id from entitiesinagroup" +
                                                      " where entity_id in (" + containingEntityIds + ") order by group_id limit " +
                                                      checkIfShouldBeAllResults(inLimit), "Long")
      rtgRows
    } else Nil
  }

  /** Intended to show something like an activity log. Could be used for someone to show their personal journal or for other reporting.
    */
  def findJournalEntries(startTimeIn: Long, endTimeIn: Long, inLimitIn: Option[Long] = None): Array[(Long, String, Long)] = {
    val rows: List[Array[Option[Any]]] = dbQuery("select insertion_date, 'Added: ' || name, id from entity where insertion_date >= " + startTimeIn +
                                                        " and insertion_date <= " + endTimeIn +
                                                 " UNION " +
                                                 "select archived_date, 'Archived: ' || name, id from entity where archived and archived_date >= " + startTimeIn +
                                                        " and archived_date <= " + endTimeIn +
                                                 " order by 1 limit " + checkIfShouldBeAllResults(inLimitIn), "Long,String,Long")
    val results = new Array[(Long, String, Long)](rows.size)
    var n = 0
    for (row <- rows) {
      results(n) = (row(0).get.asInstanceOf[Long], row(1).get.asInstanceOf[String], row(2).get.asInstanceOf[Long])
      n += 1
    }
    results
  }

  def getCountOfGroupsContainingEntity(entityIdIn: Long): Long = {
    extractRowCountFromCountQuery("select count(1) from EntitiesInAGroup where entity_id=" + entityIdIn)
  }

  def getContainingGroupsIds(entityIdIn: Long): List[Long] = {
    val groupIds: List[Array[Option[Any]]] = dbQuery("select group_id from EntitiesInAGroup where entity_id=" + entityIdIn,
                                                     "Long")
    var results: List[Long] = Nil
    for (row <- groupIds) {
      results = row(0).get.asInstanceOf[Long] :: results
    }
    results
  }

  def isEntityInGroup(inGroupId: Long, inEntityId: Long): Boolean = {
    val num = extractRowCountFromCountQuery("select count(1) from EntitiesInAGroup eig, entity e where eig.entity_id=e.id" +
                                            (if (!showAllArchivedEntities) {
                                              " and (not e.archived)"
                                            } else {
                                              ""
                                            }) +
                                            " and group_id=" + inGroupId + " and entity_id=" + inEntityId)
    if (num > 1) throw new OmDatabaseException("Entity " + inEntityId + " is in group " + inGroupId + " " + num + " times?? Should be 0 or 1.")
    num == 1
  }

  /** Before calling this, the caller should have made sure that any parameters it received in the form of
    * Strings should have been passed through escapeQuotesEtc FIRST, and ONLY THE RESULT SENT HERE.
    * Returns the # of results, and the results (a collection of rows, each row being its own collection).
    *
    * idea: probably should change the data types from List to Vector or other, once I finish reading about that.
    */
  private def dbQuery(sql: String, types: String): List[Array[Option[Any]]] = {
    // Note: pgsql docs say "Under the JDBC specification, you should access a field only once" (under the JDBC interface part).
    checkForBadSql(sql)
    // idea: results could change to a val and be filled w/ a recursive helper method; other vars might go away then too.
    var results: List[Array[Option[Any]]] = Nil
    val typesAsArray: Array[String] = types.split(",")
    var st: Statement = null
    var rs: ResultSet = null
    var rowCounter = 0
    try {
      st = mConn.createStatement
      rs = st.executeQuery(sql)
      // idea: (see comment at other use in this class, of getWarnings)
      // idea: maybe both uses of getWarnings should be combined into a method.
      val warnings = rs.getWarnings
      val warnings2 = st.getWarnings
      if (warnings != null || warnings2 != null) throw new OmDatabaseException("Warnings from postgresql. Matters? Says: " + warnings + ", and " + warnings2)
      while (rs.next) {
        rowCounter += 1
        val row: Array[Option[Any]] = new Array[Option[Any]](typesAsArray.length)
        //1-based counter for db results, but array is 0-based, so will compensate w/ -1:
        var columnCounter = 0
        for (typeString: String <- typesAsArray) {
          // the for loop is to take is through all the columns in this row, as specified by the caller in the "types" parm.
          columnCounter += 1
          if (rs.getObject(columnCounter) == null) row(columnCounter - 1) = None
          else {
            if (typeString == "Float") {
              row(columnCounter - 1) = Some(rs.getFloat(columnCounter))
            } else if (typeString == "String") {
              row(columnCounter - 1) = Some(PostgreSQLDatabase.unEscapeQuotesEtc(rs.getString(columnCounter)))
            } else if (typeString == "Long") {
              row(columnCounter - 1) = Some(rs.getLong(columnCounter))
            } else if (typeString == "Boolean") {
              row(columnCounter - 1) = Some(rs.getBoolean(columnCounter))
            } else if (typeString == "Int") {
              row(columnCounter - 1) = Some(rs.getInt(columnCounter))
            } else throw new OmDatabaseException("unexpected value: '" + typeString + "'")
          }
        }
        results = row :: results
      }
    } catch {
      case e: Exception => throw new OmDatabaseException("Exception while processing sql: " + sql, e)
    } finally {
      if (rs != null) rs.close()
      if (st != null) st.close()
    }
    require(rowCounter == results.size)
    results.reverse
  }

  def dbQueryWrapperForOneRow(sql: String, types: String): Array[Option[Any]] = {
    val results = dbQuery(sql, types)
    if (results.size != 1) throw new OmDatabaseException("Got " + results.size + " instead of 1 result from sql " + sql + "??")
    results.head
  }

  def getQuantityAttributeData(inQuantityId: Long): Array[Option[Any]] = {
    dbQueryWrapperForOneRow("select qa.entity_id, qa.unit_id, qa.quantity_number, qa.attr_type_id, qa.valid_on_date, qa.observation_date, asort.sorting_index " +
                            "from QuantityAttribute qa, AttributeSorting asort where qa.id=" + inQuantityId +
                            " and qa.entity_id=asort.entity_id and asort.attribute_form_id=" + PostgreSQLDatabase.getAttributeFormId(Controller.QUANTITY_TYPE) +
                            " and qa.id=asort.attribute_id",
                            "Long,Long,Float,Long,Long,Long,Long")
  }

  def getRelationToEntityData(inRelTypeId: Long, inEntityId1: Long, inEntityId2: Long): Array[Option[Any]] = {
    dbQueryWrapperForOneRow("select rte.id, rte.valid_on_date, rte.observation_date, asort.sorting_index" +
                            " from RelationToEntity rte, AttributeSorting asort" +
                            " where rte.rel_type_id=" + inRelTypeId + " and rte.entity_id=" + inEntityId1 + " and rte.entity_id_2=" + inEntityId2 +
                            " and rte.entity_id=asort.entity_id and asort.attribute_form_id=" + PostgreSQLDatabase.getAttributeFormId(Controller.RELATION_TO_ENTITY_TYPE) +
                            " and rte.id=asort.attribute_id",
                            "Long,Long,Long,Long")
  }

  def getAllRelationToEntityDataById(inId: Long): Array[Option[Any]] = {
    dbQueryWrapperForOneRow("select form_id, id, rel_type_id, entity_id, entity_id_2, valid_on_date, observation_date from RelationToEntity where id=" + inId,
                            "Int,Long,Long,Long,Long,Long,Long")
  }

  def getGroupData(inId: Long): Array[Option[Any]] = {
    dbQueryWrapperForOneRow("select name, insertion_date, allow_mixed_classes from grupo where id=" + inId,
                            "String,Long,Boolean")
  }

  def getRelationToGroupData(entityId: Long, relTypeId: Long, groupId: Long): Array[Option[Any]] = {
    dbQueryWrapperForOneRow("select rtg.id, rtg.entity_id, rtg.rel_type_id, rtg.group_id, rtg.valid_on_date, rtg.observation_date, asort.sorting_index " +
                            "from RelationToGroup rtg, AttributeSorting asort" +
                            " where rtg.entity_id=" + entityId + " and rtg.rel_type_id=" + relTypeId + " and rtg.group_id=" + groupId +
                            " and rtg.entity_id=asort.entity_id and asort.attribute_form_id=" + PostgreSQLDatabase.getAttributeFormId(Controller.RELATION_TO_GROUP_TYPE) +
                            " and rtg.id=asort.attribute_id",
                            "Long,Long,Long,Long,Long,Long,Long")
  }

  def getAllRelationToGroupDataById(idIn: Long): Array[Option[Any]] = {
    dbQueryWrapperForOneRow("select form_id, id, entity_id, rel_type_id, group_id, valid_on_date, observation_date from RelationToGroup " +
                            " where id=" + idIn,
                            "Int,Long,Long,Long,Long,Long,Long")
  }


  def getRelationToGroupDataById(idIn: Long): Array[Option[Any]] = {
    dbQueryWrapperForOneRow("select rtg.id, rtg.entity_id, rtg.rel_type_id, rtg.group_id, rtg.valid_on_date, rtg.observation_date, asort.sorting_index " +
                            "from RelationToGroup rtg, AttributeSorting asort" +
                            " where id=" + idIn +
                            " and rtg.entity_id=asort.entity_id and asort.attribute_form_id=" + PostgreSQLDatabase.getAttributeFormId(Controller.RELATION_TO_GROUP_TYPE) +
                            " and rtg.id=asort.attribute_id",
                            "Long,Long,Long,Long,Long,Long,Long")
  }

  def getRelationTypeData(inId: Long): Array[Option[Any]] = {
    dbQueryWrapperForOneRow("select name, name_in_reverse_direction, directionality from RelationType r, Entity e where " +
                            (if (!showAllArchivedEntities) {
                              "(not archived) and "
                            } else {
                              ""
                            }) +
                            "e.id=r.entity_id " +
                            "and r.entity_id=" +
                            inId,
                            "String,String,String")
  }

  // idea: combine all the methods that look like this (s.b. easier now, in scala, than java)
  def getTextAttributeData(inTextId: Long): Array[Option[Any]] = {
    dbQueryWrapperForOneRow("select ta.entity_id, ta.textValue, ta.attr_type_id, ta.valid_on_date, ta.observation_date, asort.sorting_index" +
                            " from TextAttribute ta, AttributeSorting asort where id=" + inTextId +
                            " and ta.entity_id=asort.entity_id and asort.attribute_form_id=" + PostgreSQLDatabase.getAttributeFormId(Controller.TEXT_TYPE) +
                            " and ta.id=asort.attribute_id",
                            "Long,String,Long,Long,Long,Long")
  }

  def getDateAttributeData(inDateId: Long): Array[Option[Any]] = {
    dbQueryWrapperForOneRow("select da.entity_id, da.date, da.attr_type_id, asort.sorting_index " +
                            "from DateAttribute da, AttributeSorting asort where da.id=" + inDateId +
                            " and da.entity_id=asort.entity_id and asort.attribute_form_id=" + PostgreSQLDatabase.getAttributeFormId(Controller.DATE_TYPE) +
                            " and da.id=asort.attribute_id",
                            "Long,Long,Long,Long")
  }

  def getBooleanAttributeData(inBooleanId: Long): Array[Option[Any]] = {
    dbQueryWrapperForOneRow("select ba.entity_id, ba.booleanValue, ba.attr_type_id, ba.valid_on_date, ba.observation_date, asort.sorting_index" +
                            " from BooleanAttribute ba, AttributeSorting asort where id=" + inBooleanId +
                            " and ba.entity_id=asort.entity_id and asort.attribute_form_id=" + PostgreSQLDatabase.getAttributeFormId(Controller.BOOLEAN_TYPE) +
                            " and ba.id=asort.attribute_id",
                            "Long,Boolean,Long,Long,Long,Long")
  }

  def getFileAttributeData(inFileId: Long): Array[Option[Any]] = {
    dbQueryWrapperForOneRow("select fa.entity_id, fa.description, fa.attr_type_id, fa.original_file_date, fa.stored_date, fa.original_file_path, fa.readable, " +
                            "fa.writable, fa.executable, fa.size, fa.md5hash, asort.sorting_index " +
                            " from FileAttribute fa, AttributeSorting asort where id=" + inFileId +
                            " and fa.entity_id=asort.entity_id and asort.attribute_form_id=" + PostgreSQLDatabase.getAttributeFormId(Controller.FILE_TYPE) +
                            " and fa.id=asort.attribute_id",
                            "Long,String,Long,Long,Long,String,Boolean,Boolean,Boolean,Long,String,Long")
  }

  def getFileAttributeContent(fileAttributeIdIn: Long, outputStreamIn: java.io.OutputStream): (Long, String) = {
    def action(bufferIn: Array[Byte], startingIndexIn: Int, numBytesIn: Int) {
      outputStreamIn.write(bufferIn, startingIndexIn, numBytesIn)
    }
    actOnFileFromServer(fileAttributeIdIn, action)
  }

  def updateEntityInAGroup(groupIdIn: Long, entityIdIn: Long, sortingIndexIn: Long) {
    dbAction("update EntitiesInAGroup set (sorting_index) = (" + sortingIndexIn + ") where group_id=" + groupIdIn + " and  " +
             "entity_id=" + entityIdIn)
  }

  def updateAttributeSorting(entityIdIn: Long, attributeFormIdIn: Long, attributeIdIn: Long, sortingIndexIn: Long) {
    dbAction("update AttributeSorting set (sorting_index) = (" + sortingIndexIn + ") where entity_id=" + entityIdIn + " and  " +
             "attribute_form_id=" + attributeFormIdIn + " and attribute_id=" + attributeIdIn)
  }

  /** Returns whether the stored and calculated md5hashes match, and an error message when they don't.
    */
  def verifyFileAttributeContentIntegrity(fileAttributeIdIn: Long): (Boolean, Option[String]) = {
    // idea: combine w/ similar logic in FileAttribute.md5Hash?
    val d = java.security.MessageDigest.getInstance("MD5")
    def action(bufferIn: Array[Byte], startingIndexIn: Int, numBytesIn: Int) {
      d.update(bufferIn, startingIndexIn, numBytesIn)
    }
    val storedMd5Hash = actOnFileFromServer(fileAttributeIdIn, action)._2
    //noinspection LanguageFeature ...It is a style violation (advanced feature) but it's what I found when searching for how to do it.
    // outputs same as command 'md5sum <file>'.
    val md5hash: String = d.digest.map(0xFF &).map {"%02x".format(_)}.foldLeft("") {_ + _}
    if (md5hash == storedMd5Hash) (true, None)
    else {
      (false, Some("Mismatched md5hashes: " + storedMd5Hash + " (stored in the md5sum db column) != " + md5hash + "(calculated from stored file contents)"))
    }
  }

  /** This is a no-op, called in actOnFileFromServer, that a test can customize to simulate a corrupted file on the server. */
  //noinspection ScalaUselessExpression (...intentional style violation, for readability)
  def damageBuffer(buffer: Array[Byte]): Unit = Unit

  /** Returns the file size (having confirmed it is the same as the # of bytes processed), and the md5hash that was stored with the document.
    */
  def actOnFileFromServer(fileAttributeIdIn: Long, actionIn: (Array[Byte], Int, Int) => Unit): (Long, String) = {
    var obj: LargeObject = null
    try {
      // even though we're not storing data, the instructions (see createTables re this...) said to have it in a transaction.
      beginTrans()
      val lobjManager: LargeObjectManager = mConn.asInstanceOf[org.postgresql.PGConnection].getLargeObjectAPI
      val oidOption: Option[Long] = dbQueryWrapperForOneRow("select contents_oid from FileAttributeContent where file_attribute_id=" + fileAttributeIdIn,
                                                            "Long")(0).asInstanceOf[Option[Long]]
      if (oidOption.isEmpty) throw new OmDatabaseException("No contents found for file attribute id " + fileAttributeIdIn)
      val oid: Long = oidOption.get
      obj = lobjManager.open(oid, LargeObjectManager.READ)
      val buffer = new Array[Byte](2048)
      var numBytesRead = 0
      var total: Long = 0
      @tailrec
      def readFileFromDbAndActOnIt() {
        numBytesRead = obj.read(buffer, 0, buffer.length)
        // (intentional style violation, for readability):
        //noinspection ScalaUselessExpression
        if (numBytesRead <= 0) Unit
        else {
          // just once by a test subclass is enough to mess w/ the md5sum.
          if (total == 0) damageBuffer(buffer)

          actionIn(buffer, 0, numBytesRead)
          total += numBytesRead
          readFileFromDbAndActOnIt()
        }
      }
      readFileFromDbAndActOnIt()
      val resultOption = dbQueryWrapperForOneRow("select size, md5hash from fileattribute where id=" + fileAttributeIdIn, "Long,String")
      if (resultOption(0).isEmpty) throw new OmDatabaseException("No result from query for fileattribute for id " + fileAttributeIdIn + ".")
      val (contentSize, md5hash) = (resultOption(0).get.asInstanceOf[Long], resultOption(1).get.asInstanceOf[String])
      if (total != contentSize) {
        throw new OmFileTransferException("Transferred " + total + " bytes instead of " + contentSize + "??")
      }
      commitTrans()
      (total, md5hash)
    } catch {
      case e: Exception => throw rollbackWithCatch(e)
    } finally {
      try {
        obj.close()
      } catch {
        case e: Exception =>
        // not sure why this fails sometimes, if it's a bad thing or not, but for now not going to be stuck on it.
        // idea: look at the source code.
      }
    }
  }

  def quantityAttributeKeyExists(idIn: Long): Boolean = doesThisExist("SELECT count(1) from QuantityAttribute where id=" + idIn)

  def textAttributeKeyExists(idIn: Long): Boolean = doesThisExist("SELECT count(1) from TextAttribute where id=" + idIn)

  def dateAttributeKeyExists(idIn: Long): Boolean = doesThisExist("SELECT count(1) from DateAttribute where id=" + idIn)

  def booleanAttributeKeyExists(idIn: Long): Boolean = doesThisExist("SELECT count(1) from BooleanAttribute where id=" + idIn)

  def fileAttributeKeyExists(idIn: Long): Boolean = doesThisExist("SELECT count(1) from FileAttribute where id=" + idIn)

  def relationToEntityKeyExists(idIn: Long): Boolean = doesThisExist("SELECT count(1) from RelationToEntity where id=" + idIn)

  def relationToGroupKeyExists(idIn: Long): Boolean = doesThisExist("SELECT count(1) from RelationToGroup where id=" + idIn)

  def attributeKeyExists(formIdIn: Long, idIn: Long): Boolean = {
      //MAKE SURE THESE MATCH WITH THOSE IN getAttributeFormId !
      formIdIn match {
        case 1 => quantityAttributeKeyExists(idIn)
        case 2 => dateAttributeKeyExists(idIn)
        case 3 => booleanAttributeKeyExists(idIn)
        case 4 => fileAttributeKeyExists(idIn)
        case 5 => textAttributeKeyExists(idIn)
        case 6 => relationToEntityKeyExists(idIn)
        case 7 => relationToGroupKeyExists(idIn)
      }
  }

  /** Excludes those entities that are really relationtypes, attribute types, or quantity units. */
  def entityOnlyKeyExists(inID: Long): Boolean = {
    doesThisExist("SELECT count(1) from Entity where " +
                  (if (!showAllArchivedEntities) "(not archived) and " else "") +
                  "id=" + inID + " and id in (select id from entity " + limitToEntitiesOnly(ENTITY_ONLY_SELECT_PART) + ")")
  }

  def entityKeyExists(inID: Long, includeArchived: Boolean = true): Boolean = {
    val condition = if (!includeArchived && ! showAllArchivedEntities) " and not archived" else ""
    doesThisExist("SELECT count(1) from Entity where id=" + inID + condition)
  }

  def groupEntrySortingIndexInUse(groupIdIn: Long, sortingIndexIn: Long): Boolean = doesThisExist("SELECT count(1) from Entitiesinagroup where group_id=" +
                                                                                                  groupIdIn + " and sorting_index=" + sortingIndexIn)

  def attributeSortingIndexInUse(entityIdIn: Long, sortingIndexIn: Long): Boolean = doesThisExist("SELECT count(1) from AttributeSorting where entity_id=" +
                                                                                                  entityIdIn + " and sorting_index=" + sortingIndexIn)

  def classKeyExists(inID: Long): Boolean = doesThisExist("SELECT count(1) from class where id=" + inID)

  def relationTypeKeyExists(inId: Long): Boolean = doesThisExist("SELECT count(1) from RelationType where entity_id=" + inId)

  def relationToEntityKeysExistAndMatch(idIn: Long, relTypeIdIn: Long, entityId1In: Long, entityId2In: Long): Boolean = {
    doesThisExist("SELECT count(1) from RelationToEntity where id=" + idIn + " and rel_type_id=" + relTypeIdIn + " and entity_id=" + entityId1In +
                  " and entity_id_2=" + entityId2In)
  }

  def relationToEntityExists(relTypeIdIn: Long, entityId1In: Long, entityId2In: Long): Boolean = {
    doesThisExist("SELECT count(1) from RelationToEntity where rel_type_id=" + relTypeIdIn + " and entity_id=" + entityId1In +
                  " and entity_id_2=" + entityId2In)
  }

  def groupKeyExists(inId: Long): Boolean = {
    doesThisExist("SELECT count(1) from grupo where id=" + inId)
  }

  def relationToGroupKeysExistAndMatch(id: Long, entityId: Long, relTypeId: Long, groupId: Long): Boolean = {
    doesThisExist("SELECT count(1) from RelationToGroup where id=" + id + " and entity_id=" + entityId + " and rel_type_id=" + relTypeId +
                  " and group_id=" + groupId)
  }

  /**
   * Allows querying for a range of objects in the database; returns a java.util.Map with keys and names.
   * 1st parm is index to start with (0-based), 2nd parm is # of obj's to return (if None, means no limit).
   */
  def getEntities(inStartingObjectIndex: Long, inMaxVals: Option[Long] = None): java.util.ArrayList[Entity] = {
    getEntitiesGeneric(inStartingObjectIndex, inMaxVals, Controller.ENTITY_TYPE)
  }

  /** Excludes those entities that are really relationtypes, attribute types, or quantity units. Otherwise similar to getEntities.
    *
    * *****NOTE*****: The limitByClass:Boolean parameter is not redundant with the inClassId: inClassId could be None and we could still want
    * to select only those entities whose class_id is NULL, such as when enforcing group uniformity (see method hasMixedClasses and its
    * uses, for more info).
    *
    * The parameter omitEntity is (at this writing) used for the id of a class-defining (template) entity, which we shouldn't show for editing when showing all the
    * entities in the class (editing that is a separate menu option), otherwise it confuses things.
    * */
  def getEntitiesOnly(inStartingObjectIndex: Long, inMaxVals: Option[Long] = None, inClassId: Option[Long] = None,
                      limitByClass: Boolean = false, templateEntity: Option[Long] = None,
                      groupToOmitIdIn: Option[Long] = None): java.util.ArrayList[Entity] = {
    getEntitiesGeneric(inStartingObjectIndex, inMaxVals, "EntityOnly", inClassId, limitByClass, templateEntity, groupToOmitIdIn)
  }

  /** similar to getEntities */
  def getRelationTypes(inStartingObjectIndex: Long, inMaxVals: Option[Long] = None): java.util.ArrayList[Entity] = {
    getEntitiesGeneric(inStartingObjectIndex, inMaxVals, Controller.RELATION_TYPE_TYPE)
  }

  def getMatchingEntities(inStartingObjectIndex: Long, inMaxVals: Option[Long] = None, omitEntityIdIn: Option[Long],
                          nameRegexIn: String): java.util.ArrayList[Entity] = {
    val nameRegex = escapeQuotesEtc(nameRegexIn)
    val omissionExpression: String = if (omitEntityIdIn.isEmpty) "true" else "(not id=" + omitEntityIdIn.get + ")"
    val sql: String = "select id, name, class_id, insertion_date, public, archived from entity where " +
                      (if (!showAllArchivedEntities) {
                        "not archived and "
                      } else {
                        ""
                      }) +
                      omissionExpression +
                      " and name ~* '" + nameRegex + "'" +
                      " UNION " +
                      "select id, name, class_id, insertion_date, public, archived from entity where " +
                      (if (!showAllArchivedEntities) {
                        "not archived and "
                      } else {
                        ""
                      }) +
                      omissionExpression +
                      " and id in (select entity_id from textattribute where textValue ~* '" + nameRegex + "')" +
                      " ORDER BY" +
                      " id limit " + checkIfShouldBeAllResults(inMaxVals) + " offset " + inStartingObjectIndex
    val earlyResults = dbQuery(sql, "Long,String,Long,Long,Boolean,Boolean")
    val finalResults = new java.util.ArrayList[Entity]
    // idea: (see getEntitiesGeneric for idea, see if applies here)
    for (result <- earlyResults) {
      // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
      finalResults.add(new Entity(this, result(0).get.asInstanceOf[Long], result(1).get.asInstanceOf[String], result(2).asInstanceOf[Option[Long]],
                                  result(3).get.asInstanceOf[Long], result(4).asInstanceOf[Option[Boolean]], result(5).get.asInstanceOf[Boolean]))
    }
    require(finalResults.size == earlyResults.size)
    finalResults
  }

  def getMatchingGroups(inStartingObjectIndex: Long, inMaxVals: Option[Long] = None, omitGroupIdIn: Option[Long],
                        nameRegexIn: String): java.util.ArrayList[Group] = {
    val nameRegex = escapeQuotesEtc(nameRegexIn)
    val omissionExpression: String = if (omitGroupIdIn.isEmpty) "true" else "(not id=" + omitGroupIdIn.get + ")"
    val sql: String = s"select id, name, insertion_date, allow_mixed_classes from grupo where name ~* '$nameRegex'" +
                      " and " + omissionExpression + " order by id limit " + checkIfShouldBeAllResults(inMaxVals) + " offset " + inStartingObjectIndex
    val earlyResults = dbQuery(sql, "Long,String,Long,Boolean")
    val finalResults = new java.util.ArrayList[Group]
    // idea: (see getEntitiesGeneric for idea, see if applies here)
    for (result <- earlyResults) {
      // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
      finalResults.add(new Group(this, result(0).get.asInstanceOf[Long], result(1).get.asInstanceOf[String], result(2).get.asInstanceOf[Long],
                                 result(3).get.asInstanceOf[Boolean]))
    }
    require(finalResults.size == earlyResults.size)
    finalResults
  }

  def getContainingEntities_helper(sqlIn: String): java.util.ArrayList[(Long, Entity)] = {
    val earlyResults = dbQuery(sqlIn, "Long,Long")
    val finalResults = new java.util.ArrayList[(Long, Entity)]
    // idea: should the remainder of this method be moved to Entity, so the persistence layer doesn't know anything about the Model? (helps avoid circular
    // dependencies? is a cleaner design?.)
    for (result <- earlyResults) {
      // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
      val rel_type_id: Long = result(0).get.asInstanceOf[Long]
      val entity: Entity = new Entity(this, result(1).get.asInstanceOf[Long])
      finalResults.add((rel_type_id, entity))
    }

    require(finalResults.size == earlyResults.size)
    finalResults
  }

  def getEntitiesContainingEntity(entityIn: Entity, startingIndexIn: Long, maxValsIn: Option[Long] = None): java.util.ArrayList[(Long, Entity)] = {
    val sql: String = "select rel_type_id, entity_id from relationtoentity where entity_id_2=" + entityIn.getId + " order by entity_id limit " +
                      checkIfShouldBeAllResults(maxValsIn) + " offset " + startingIndexIn
    //note/idea: this should be changed when we update relation stuff similarly, to go both ways in the relation (either entity_id or
    // entity_id_2: helpfully returned; & in UI?)
    //And, perhaps changed to account for whether something is archived.
    // See getCountOfEntitiesContainingEntity for example.
    getContainingEntities_helper(sql)
  }

  def getEntitiesContainingGroup(groupIdIn: Long, startingIndexIn: Long, maxValsIn: Option[Long] = None): java.util.ArrayList[(Long, Entity)] = {
    val sql: String = "select rel_type_id, entity_id from relationtogroup where group_id=" + groupIdIn +
                      " order by entity_id, rel_type_id limit " +
                      checkIfShouldBeAllResults(maxValsIn) + " offset " + startingIndexIn
    //note/idea: this should be changed when we update relation stuff similarly, to go both ways in the relation (either entity_id or
    // entity_id_2: helpfully returned; & in UI?)
    //And, perhaps changed to account for whether something is archived.
    // See getCountOfEntitiesContainingGroup for example.
    getContainingEntities_helper(sql)
  }

  /**
   * @return A tuple showing the # of non-archived entities and the # of archived entities that directly refer to this entity (IN *ONE* DIRECTION ONLY).
   */
  def getCountOfEntitiesContainingEntity(entityIdIn: Long): (Long, Long) = {
    val nonArchived2 = extractRowCountFromCountQuery("select count(1) from relationtoentity rte, entity e where e.id=rte.entity_id_2 and not e.archived" +
                                                     " and e.id=" + entityIdIn)
    val archived2 = extractRowCountFromCountQuery("select count(1) from relationtoentity rte, entity e where e.id=rte.entity_id_2 and e.archived" +
                                                  " and e.id=" + entityIdIn)

    (nonArchived2, archived2)
  }

  /**
   * @return A tuple showing the # of non-archived entities and the # of archived entities that directly refer to this group.
   */
  def getCountOfEntitiesContainingGroup(groupIdIn: Long): (Long, Long) = {
    val nonArchived = extractRowCountFromCountQuery("select count(1) from relationtogroup rtg, entity e where e.id=rtg.entity_id and not e.archived" +
                                                    " and rtg.group_id=" + groupIdIn)
    val archived = extractRowCountFromCountQuery("select count(1) from relationtogroup rtg, entity e where e.id=rtg.entity_id and e.archived" +
                                                 " and rtg.group_id=" + groupIdIn)
    (nonArchived, archived)
  }

  def getContainingRelationToGroups(entityIn: Entity, startingIndexIn: Long, maxValsIn: Option[Long] = None): java.util.ArrayList[RelationToGroup] = {
    // BUG (tracked in tasks): there is a disconnect here between this method and its _helper method, because one uses the eig table, the other the rtg table,
    // and there is no requirement/enforcement that all groups defined in eig are in an rtg, so they could get dif't/unexpected results.
    // So, could: see the expectation of the place(s) calling this method, if uniform, make these 2 methods more uniform in what they do in meeting that,
    // OR, could consider whether we really should have an enforcement between the 2 tables...?
    // THIS BUg currently prevents searching for then deleting the entity w/ this in name: "OTHER ENTITY NOTED IN A DELETION BUG" (see also other issue
    // in Controller.java where that same name is mentioned. Related, be cause in that case on the line:
    //    "descriptions = descriptions.substring(0, descriptions.length - delimiter.length) + ".  ""
    // ...one gets the below exception throw, probably for the same or related reason:
        /*
        ==============================================
        **CURRENT ENTITY:while at it, order a valentine's card on amazon asap (or did w/ cmas shopping?)
        No attributes have been assigned to this object, yet.
        1-Add attribute (quantity, true/false, date, text, external file, relation to entity or group: i.e., ownership of or "has" another entity, family ties, etc)...
        2-Import/Export...
        3-Edit name
        4-Delete or Archive...
        5-Go to...
        6-List next items
        7-Set current entity (while at it, order a valentine's card on amazon asap (or did w/ cmas shopping?)) as default (first to come up when launching this program.)
        8-Edit public/nonpublic status
        0/ESC - back/previous menu
        4






        ==============================================
        Choose a deletion or archiving option:
        1-Delete this entity
                 2-Archive this entity (remove from visibility but not permanent/total deletion)
        0/ESC - back/previous menu
        1
        An error occurred: "java.lang.StringIndexOutOfBoundsException: String index out of range: -2".  If you can provide simple instructions to reproduce it consistently, maybe it can be fixed.  Do you want to see the detailed output? (y/n):
          y






        ==============================================
        java.lang.StringIndexOutOfBoundsException: String index out of range: -2
        at java.lang.String.substring(String.java:1911)
        at org.onemodel.controller.Controller.deleteOrArchiveEntity(Controller.scala:644)
        at org.onemodel.controller.EntityMenu.entityMenu(EntityMenu.scala:232)
        at org.onemodel.controller.EntityMenu.entityMenu(EntityMenu.scala:388)
        at org.onemodel.controller.Controller.showInEntityMenuThenMainMenu(Controller.scala:277)
        at org.onemodel.controller.MainMenu.mainMenu(MainMenu.scala:80)
        at org.onemodel.controller.MainMenu.mainMenu(MainMenu.scala:98)
        at org.onemodel.controller.MainMenu.mainMenu(MainMenu.scala:98)
        at org.onemodel.controller.Controller.menuLoop$1(Controller.scala:140)
        at org.onemodel.controller.Controller.start(Controller.scala:143)
        at org.onemodel.TextUI.launchUI(TextUI.scala:220)
        at org.onemodel.TextUI$.main(TextUI.scala:34)
        at org.onemodel.TextUI.main(TextUI.scala:1)
        */

    val sql: String = "select group_id from entitiesinagroup where entity_id=" + entityIn.getId + " order by group_id limit " +
                      checkIfShouldBeAllResults(maxValsIn) + " offset " + startingIndexIn
    getContainingRelationToGroups_helper(sql)
  }

  def getContainingRelationToGroups_helper(sqlIn: String): java.util.ArrayList[RelationToGroup] = {
    val earlyResults = dbQuery(sqlIn, "Long")
    val groupIdResults = new java.util.ArrayList[Long]
    // idea: should the remainder of this method be moved to Group, so the persistence layer doesn't know anything about the Model? (helps avoid circular
    // dependencies? is a cleaner design?)
    for (result <- earlyResults) {
      //val group:Group = new Group(this, result(0).asInstanceOf[Long])
      groupIdResults.add(result(0).get.asInstanceOf[Long])
    }
    require(groupIdResults.size == earlyResults.size)
    val containingRelationToGroups: java.util.ArrayList[RelationToGroup] = new java.util.ArrayList[RelationToGroup]
    for (gid <- groupIdResults.toArray) {
      val rtgs = getRelationToGroupsByGroup(gid.asInstanceOf[Long], 0)
      for (rtg <- rtgs.toArray) containingRelationToGroups.add(rtg.asInstanceOf[RelationToGroup])
    }
    containingRelationToGroups
  }

  // 1st parm is 0-based index to start with, 2nd parm is # of obj's to return (if None, means no limit).
  private def getEntitiesGeneric(inStartingObjectIndex: Long, inMaxVals: Option[Long], inTableName: String,
                                 inClassId: Option[Long] = None, limitByClass: Boolean = false,
                                 templateEntity: Option[Long] = None, groupToOmitIdIn: Option[Long] = None): java.util.ArrayList[Entity] = {
    val ENTITY_SELECT_PART: String = "SELECT e.id, e.name, e.class_id, e.insertion_date, e.public, e.archived"
    val sql: String = ENTITY_SELECT_PART +
                      (if (inTableName.compareToIgnoreCase(Controller.RELATION_TYPE_TYPE) == 0) ", r.name_in_reverse_direction, r.directionality " else "") +
                      " from Entity e " +
                      (if (inTableName.compareToIgnoreCase(Controller.RELATION_TYPE_TYPE) == 0) {
                        // for RelationTypes, hit both tables since one "inherits", but limit it to those rows
                        // for which a RelationType row also exists.
                        ", RelationType r "
                      } else "") +
                      " where" +
                      (if (!showAllArchivedEntities) {
                        " (not archived) and"
                      } else {
                        ""
                      }) +
                      " true " +
                      classLimit(limitByClass, inClassId) +
                      (if (limitByClass && templateEntity.isDefined) " and id != " + templateEntity.get else "") +
                      (if (inTableName.compareToIgnoreCase(Controller.RELATION_TYPE_TYPE) == 0) {
                        // for RelationTypes, hit both tables since one "inherits", but limit it to those rows
                        // for which a RelationType row also exists.
                        " and e.id = r.entity_id "
                      } else "") +
                      (if (inTableName.compareToIgnoreCase("EntityOnly") == 0) limitToEntitiesOnly(ENTITY_SELECT_PART) else "") +
                      (if (groupToOmitIdIn.isDefined) " except (" + ENTITY_SELECT_PART + " from entity e, " +
                                                    "EntitiesInAGroup eiag where e.id=eiag.entity_id and " +
                                                    "group_id=" + groupToOmitIdIn.get + ")"
                      else "") +
                      " order by id limit " + checkIfShouldBeAllResults(inMaxVals) + " offset " + inStartingObjectIndex
    val earlyResults = dbQuery(sql,
                               if (inTableName.compareToIgnoreCase(Controller.RELATION_TYPE_TYPE) == 0) {
                                 "Long,String,Long,Long,Boolean,Boolean,String,String"
                               } else {
                                 "Long,String,Long,Long,Boolean,Boolean"
                               })
    val finalResults = new java.util.ArrayList[Entity]
    // idea: should the remainder of this method be moved to Entity, so the persistence layer doesn't know anything about the Model? (helps avoid circular
    // dependencies; is a cleaner design.)
    for (result <- earlyResults) {
      // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
      if (inTableName.compareToIgnoreCase(Controller.RELATION_TYPE_TYPE) == 0) {
        finalResults.add(new RelationType(this, result(0).get.asInstanceOf[Long], result(1).get.asInstanceOf[String], result(6).get.asInstanceOf[String],
                                          result(7).get.asInstanceOf[String]))
      } else {
        finalResults.add(new Entity(this, result(0).get.asInstanceOf[Long], result(1).get.asInstanceOf[String], result(2).asInstanceOf[Option[Long]],
                                    result(3).get.asInstanceOf[Long], result(4).asInstanceOf[Option[Boolean]], result(5).get.asInstanceOf[Boolean]))
      }
    }

    require(finalResults.size == earlyResults.size)
    finalResults
  }

  /** Allows querying for a range of objects in the database; returns a java.util.Map with keys and names.
    * 1st parm is index to start with (0-based), 2nd parm is # of obj's to return (if None, means no limit).
    */
  def getGroups(inStartingObjectIndex: Long, inMaxVals: Option[Long] = None, groupToOmitIdIn: Option[Long] = None): java.util.ArrayList[Group] = {
    val sql = "SELECT id, name, insertion_date, allow_mixed_classes from grupo " +
              " order by id limit " + checkIfShouldBeAllResults(inMaxVals) + " offset " + inStartingObjectIndex
    val earlyResults = dbQuery(sql, "Long,String,Long,Boolean")
    val finalResults = new java.util.ArrayList[Group]
    // idea: should the remainder of this method be moved to RTG, so the persistence layer doesn't know anything about the Model? (helps avoid circular
    // dependencies; is a cleaner design.)
    for (result <- earlyResults) {
      // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
      finalResults.add(new Group(this, result(0).get.asInstanceOf[Long], result(1).get.asInstanceOf[String], result(2).get.asInstanceOf[Long],
                                 result(3).get.asInstanceOf[Boolean]))
    }
    require(finalResults.size == earlyResults.size)
    finalResults
  }


  def getClasses(inStartingObjectIndex: Long, inMaxVals: Option[Long] = None): java.util.ArrayList[EntityClass] = {
    val sql: String = "SELECT id, name, defining_entity_id, create_default_attributes from class order by id limit " +
                      checkIfShouldBeAllResults(inMaxVals) + " offset " + inStartingObjectIndex
    val earlyResults = dbQuery(sql, "Long,String,Long,Boolean")
    val finalResults = new java.util.ArrayList[EntityClass]
    // idea: should the remainder of this method be moved to EntityClass, so the persistence layer doesn't know anything about the Model? (helps avoid circular
    // dependencies; is a cleaner design; see similar comment in getEntitiesGeneric.)
    for (result <- earlyResults) {
      // Only one of these values should be of "None" type, so not checking the others for that. If they are it's a bug:
      finalResults.add(new EntityClass(this, result(0).get.asInstanceOf[Long], result(1).get.asInstanceOf[String], result(2).get.asInstanceOf[Long],
                                       if (result(3).isEmpty) None else Some(result(3).get.asInstanceOf[Boolean])))
    }
    require(finalResults.size == earlyResults.size)
    finalResults
  }

  private def checkIfShouldBeAllResults(inMaxVals: Option[Long]): String = {
    if (inMaxVals.isEmpty) "ALL"
    else if (inMaxVals.get <= 0) "1"
    else inMaxVals.get.toString
  }

  def getGroupEntriesData(groupIdIn: Long, limitIn: Option[Long] = None, includeArchivedEntitiesIn: Boolean = true): List[Array[Option[Any]]] = {
    // LIKE THE OTHER 3 BELOW SIMILAR METHODS:
    // Need to make sure it gets the desired rows, rather than just some, so the order etc matters at each step, probably.
    // idea: needs automated tests (in task list also).
    var sql: String = "select eiag.entity_id, eiag.sorting_index from entity e, entitiesinagroup eiag where e.id=eiag.entity_id" +
                          " and eiag.group_id=" + groupIdIn
    if (!includeArchivedEntitiesIn && !showAllArchivedEntities) sql += " and (not e.archived)"
    sql += " order by eiag.sorting_index, eiag.entity_id limit " + checkIfShouldBeAllResults(limitIn)
    val results = dbQuery(sql, "Long,Long")
    results
  }

  def getEntityAttributeSortingData(entityIdIn: Long, limitIn: Option[Long] = None): List[Array[Option[Any]]] = {
    // see comments in getGroupEntriesData
    val results = dbQuery("select attribute_form_id, attribute_id, sorting_index from AttributeSorting where entity_id = " + entityIdIn +
                          " order by sorting_index limit " + checkIfShouldBeAllResults(limitIn),
                          "Int,Long,Long")
    results
  }

  /** As of 2014-8-4, this is only called when calculating a new sorting_index, but if it were used for something else ever, one might consider whether
    * to (optionally!) add back the code removed today which ignores archived entries.  We can't ignore them for getting a new sorting_index: bug.
    */
  def getAdjacentGroupEntriesSortingIndexes(groupIdIn: Long, sortingIndexIn: Long, limitIn: Option[Long] = None,
                                            forwardNotBackIn: Boolean): List[Array[Option[Any]]] = {
    // see comments in getGroupEntriesData.
    // Doing "not e.archived", because the caller is probably trying to move entries up/down in the UI, and if we count archived entries,
    // we could move relative to invisible entries only, and not make a visible move,
    val results = dbQuery("select eiag.sorting_index from entity e, entitiesinagroup eiag where e.id=eiag.entity_id" +
                          (if (!showAllArchivedEntities) {
                            " and (not e.archived)"
                          } else {
                            ""
                          }) +
                          " and eiag.group_id=" + groupIdIn + " and eiag.sorting_index " + (if (forwardNotBackIn) ">" else "<") + sortingIndexIn +
                          " order by eiag.sorting_index " + (if (forwardNotBackIn) "ASC" else "DESC") + ", eiag.entity_id " +
                          " limit " + checkIfShouldBeAllResults(limitIn),
                          "Long")
    results
  }

  def getAdjacentAttributesSortingIndexes(entityIdIn: Long, sortingIndexIn: Long, limitIn: Option[Long], forwardNotBackIn: Boolean): List[Array[Option[Any]]] = {
    val results = dbQuery("select sorting_index from AttributeSorting where entity_id=" + entityIdIn +
                          " and sorting_index" + (if (forwardNotBackIn) ">" else "<") + sortingIndexIn +
                          " order by sorting_index " + (if (forwardNotBackIn) "ASC" else "DESC") +
                          " limit " + checkIfShouldBeAllResults(limitIn),
                          "Long")
    results
  }

  /** This one should explicitly NOT omit archived entities (unless parameterized for that later). See caller's comments for more, on purpose.
    */
  def getNearestGroupEntrysSortingIndex(groupIdIn: Long, startingPointSortingIndexIn: Long, /* farNewNeighborSortingIndexIn: Long,*/
                                        forwardNotBackIn: Boolean): Option[Long] = {
    val results = dbQuery("select sorting_index from entitiesinagroup where group_id=" + groupIdIn + " and sorting_index " +
                          (if (forwardNotBackIn) ">" else "<") + startingPointSortingIndexIn +
                          " order by sorting_index " + (if (forwardNotBackIn) "ASC" else "DESC") +
                          " limit 1",
                          "Long")
    if (results.isEmpty) {
      None
    } else {
      if (results.size > 1) throw new OmDatabaseException("Probably the caller didn't expect this to get >1 results...Is that even meaningful?")
      else results.head(0).asInstanceOf[Option[Long]]
    }
  }

  def getNearestAttributeEntrysSortingIndex(entityIdIn: Long, startingPointSortingIndexIn: Long, forwardNotBackIn: Boolean): Option[Long] = {
    val results: List[Array[Option[Any]]] = getAdjacentAttributesSortingIndexes(entityIdIn, startingPointSortingIndexIn, Some(1), forwardNotBackIn = forwardNotBackIn)
    if (results.isEmpty) {
      None
    } else {
      if (results.size > 1) throw new OmDatabaseException("Probably the caller didn't expect this to get >1 results...Is that even meaningful?")
      else results.head(0).asInstanceOf[Option[Long]]
    }
  }

  // 2nd parm is 0-based index to start with, 3rd parm is # of obj's to return (if < 1 then it means "all"):
  def getGroupEntryObjects(inGroupId: Long, inStartingObjectIndex: Long, inMaxVals: Option[Long] = None): java.util.ArrayList[Entity] = {
    // see comments in getGroupEntriesData
    val sql = "select entity_id, sorting_index from entity e, EntitiesInAGroup eiag where e.id=eiag.entity_id" +
              (if (!showAllArchivedEntities) {
                " and (not e.archived) "
              } else {
                ""
              }) +
              " and eiag.group_id=" + inGroupId +
              " order by eiag.sorting_index, eiag.entity_id limit " + checkIfShouldBeAllResults(inMaxVals) + " offset " + inStartingObjectIndex
    val earlyResults = dbQuery(sql, "Long,Long")
    val finalResults = new java.util.ArrayList[Entity]
    // idea: should the remainder of this method be moved to Entity, so the persistence layer doesn't know anything about the Model? (helps avoid circular
    // dependencies; is a cleaner design. Or, maybe this class and all the object classes like Entity, etc, are all part of the same layer.)
    for (result <- earlyResults) {
      // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
      finalResults.add(new Entity(this, result(0).get.asInstanceOf[Long]))
    }
    require(finalResults.size == earlyResults.size)
    finalResults
  }

  private def limitToEntitiesOnly(selectColumnNames: String): String = {
    val sql: StringBuilder = new StringBuilder
    sql.append("except (").append(selectColumnNames).append(" from entity e, quantityattribute q where e.id=q.unit_id) ")
    sql.append("except (").append(selectColumnNames).append(" from entity e, quantityattribute q where e.id=q.attr_type_id) ")
    sql.append("except (").append(selectColumnNames).append(" from entity e, textattribute t where e.id=t.attr_type_id) ")
    sql.append("except (").append(selectColumnNames).append(" from entity e, relationtype t where e.id=t.entity_id) ")
    sql.append("except (").append(selectColumnNames).append(" from entity e, dateattribute t where e.id=t.attr_type_id) ")
    sql.append("except (").append(selectColumnNames).append(" from entity e, booleanattribute t where e.id=t.attr_type_id) ")
    sql.append("except (").append(selectColumnNames).append(" from entity e, fileattribute t where e.id=t.attr_type_id) ")
    sql.toString()
  }

  def getEntityData(idIn: Long): Array[Option[Any]] = {
    dbQueryWrapperForOneRow("SELECT name, class_id, insertion_date, public, archived from Entity where id=" + idIn, "String,Long,Long,Boolean,Boolean")
  }

  def getEntityName(idIn: Long): Option[String] = {
    val name: Option[Any] = getEntityData(idIn)(0)
    if (name.isEmpty) None
    else name.asInstanceOf[Option[String]]
  }

  def getClassData(idIn: Long): Array[Option[Any]] = {
    dbQueryWrapperForOneRow("SELECT name, defining_entity_id, create_default_attributes from class where id=" + idIn, "String,Long,Boolean")
  }

  def getClassName(idIn: Long): Option[String] = {
    val name: Option[Any] = getClassData(idIn)(0)
    if (name.isEmpty) None
    else name.asInstanceOf[Option[String]]
  }

  /**
   * @return the create_default_attributes boolean value from a given class.
   */
  def getClassCreateDefaultAttributes(classIdIn: Long): Option[Boolean] = {
    val value: Option[Any]= getClassData(classIdIn)(2)
    if (value.isEmpty) None
    else value.asInstanceOf[Option[Boolean]]
  }

  /**
   * @return the create_default_attributes boolean value from a given class.
   */
  def updateClassCreateDefaultAttributes(classIdIn: Long, value: Option[Boolean]) {
    dbAction("update class set (create_default_attributes) = (" +
             (if (value.isEmpty) "NULL" else if (value.get) "true" else "false") +
             ") where id=" + classIdIn)
  }

  def getTextEditorCommand: String = {
    val systemEntityId = getSystemEntityId
    val hasRelationTypeId: Long = findRelationType(PostgreSQLDatabase.theHASrelationTypeName, Some(1))(0)
    val editorInfoSystemEntity: Entity = getEntitiesFromRelationsToEntity(systemEntityId, PostgreSQLDatabase.EDITOR_INFO_ENTITY_NAME,
                                                                          Some(hasRelationTypeId), Some(1))(0)
    val textEditorInfoSystemEntity: Entity = getEntitiesFromRelationsToEntity(editorInfoSystemEntity.getId,
                                                                              PostgreSQLDatabase.TEXT_EDITOR_INFO_ENTITY_NAME, Some(hasRelationTypeId),
                                                                              Some(1))(0)
    val textEditorCommandNameAttrType: Entity = getEntitiesFromRelationsToEntity(textEditorInfoSystemEntity.getId,
                                                                         PostgreSQLDatabase.TEXT_EDITOR_COMMAND_ATTRIBUTE_TYPE_NAME, Some(hasRelationTypeId),
                                                                         Some(1))(0)
    val ta: TextAttribute = getTextAttributeByTypeId(textEditorInfoSystemEntity.getId, textEditorCommandNameAttrType.getId, Some(1))(0)
    ta.getText
  }

  def getEntitiesFromRelationsToEntity(parentEntityIdIn: Long, nameIn: String, relTypeIdIn: Option[Long] = None,
                                     expectedRows: Option[Int] = None): Array[Entity] = {
    // (not getting all the attributes in this case, and doing another query to the entity table (less efficient), to save programming
    // time for the case that the entity table changes, we don't have to carefully update all the columns selected here & the mappings.  This is a more
    // likely change than for the TextAttribute table, below.
    val queryResults: List[Array[Option[Any]]] = dbQuery("select id from entity where name='" + nameIn + "' and id in " +
                                                     "(select entity_id_2 from relationToEntity where entity_id=" + parentEntityIdIn +
                                                    (if (relTypeIdIn.isDefined) " and rel_type_id=" + relTypeIdIn.get + " " else "") + ")",
                                                    "Long")
    if (expectedRows.isDefined) {
      val count = queryResults.size
      if (count != expectedRows.get) throw new OmDatabaseException("Found " + count + " rows instead of expected " + expectedRows.get)
    }
    val finalResult = new Array[Entity](queryResults.size)
    var index = 0
    for (r <- queryResults) {
      val id: Long = r(0).get.asInstanceOf[Long]
      finalResult(index) = new Entity(this, id)
      index += 1
    }
    finalResult
  }

  def getTextAttributeByTypeId(parentEntityIdIn: Long, typeIdIn: Long, expectedRows: Option[Int] = None): Array[TextAttribute] = {
    val sql = "select ta.id, ta.textValue, ta.attr_type_id, ta.valid_on_date, ta.observation_date, asort.sorting_index " +
              " from textattribute ta, AttributeSorting asort where ta.entity_id=" + parentEntityIdIn + " and ta.attr_type_id="+typeIdIn +
              " and ta.entity_id=asort.entity_id and asort.attribute_form_id=" + PostgreSQLDatabase.getAttributeFormId(Controller.TEXT_TYPE) +
              " and ta.id=asort.attribute_id"
    val queryResults: List[Array[Option[Any]]] = dbQuery(sql, "Long,String,Long,Long,Long,Long")
    if (expectedRows.isDefined) {
      val count = queryResults.size
      if (count != expectedRows.get) throw new OmDatabaseException("Found " + count + " rows instead of expected " + expectedRows.get)
    }
    val finalResult = new Array[TextAttribute](queryResults.size)
    var index = 0
    for (r <- queryResults) {
      val textAttributeId: Long = r(0).get.asInstanceOf[Long]
      val textValue: String = r(1).get.asInstanceOf[String]
      val attrTypeId: Long = r(2).get.asInstanceOf[Long]
      val validOnDate: Option[Long] = if (r(3).isEmpty) None else Some(r(3).get.asInstanceOf[Long])
      val observationDate: Long = r(4).get.asInstanceOf[Long]
      val sortingIndex: Long = r(5).get.asInstanceOf[Long]
      finalResult(index) = new TextAttribute(this, textAttributeId, parentEntityIdIn, attrTypeId, textValue, validOnDate, observationDate, sortingIndex)
      index += 1
    }
    finalResult
  }

  /** Returns an array of tuples, each of which is of (sortingIndex, Attribute), and a Long indicating the total # that could be returned with
    * infinite display space (total existing).
    *
    * The parameter maxValsIn can be 0 for 'all'.
    *
    * Idea to improve efficiency: make this able to query only those attributes needed to satisfy the maxValsIn parameter (by first checking
    * the AttributeSorting table).  In other words, no need to read all 1500 attributes to display on the screen, just to know which ones come first, if
    * only 10 can be displayed right now and the rest might not need to be displayed.  Because right now, we have to query all data from the AttributeSorting
    * table, then all attributes (since remember they might not *be* in the AttributeSorting table), then sort them with the best available information,
    * then decide which ones to return.  Maybe instead we could do that smartly, on just the needed subset.  But it still need to gracefully handle it
    * when a given attribute (or all) is not found in the sorting table.
    */
  def getSortedAttributes(inEntityId: Long, inStartingObjectIndex: Int = 0, maxValsIn: Int = 0): (Array[(Long, Attribute)], Int) = {
    val allResults: java.util.ArrayList[(Option[Long], Attribute)] = new java.util.ArrayList[(Option[Long], Attribute)]
    // First select the counts from each table, keep a running total so we know when to select attributes (compared to inStartingObjectIndex)
    // and when to stop.
    val tables: Array[String] = Array(Controller.QUANTITY_TYPE, Controller.BOOLEAN_TYPE, Controller.DATE_TYPE, Controller.TEXT_TYPE, Controller.FILE_TYPE, Controller.RELATION_TO_ENTITY_TYPE,
                                      Controller.RELATION_TO_GROUP_TYPE)
    val columnsSelectedByTable: Array[String] = Array("id,entity_id,attr_type_id,unit_id,quantity_number,valid_on_date,observation_date",
                                                      "id,entity_id,attr_type_id,booleanValue,valid_on_date,observation_date",
                                                      "id,entity_id,attr_type_id,date",
                                                      "id,entity_id,attr_type_id,textValue,valid_on_date,observation_date",

                                                      "id,entity_id,attr_type_id,description,original_file_date,stored_date,original_file_path,readable," +
                                                      "writable,executable,size,md5hash",

                                                      "id,rel_type_id,entity_id,entity_id_2,valid_on_date,observation_date",
                                                      "id,entity_id,rel_type_id,group_id,valid_on_date,observation_date")
    val typesByTable: Array[String] = Array("Long,Long,Long,Long,Long,Float,Long,Long",
                                            "Long,Long,Long,Long,Boolean,Long,Long",
                                            "Long,Long,Long,Long,Long",
                                            "Long,Long,Long,Long,String,Long,Long",
                                            "Long,Long,Long,Long,String,Long,Long,String,Boolean,Boolean,Boolean,Long,String",
                                            "Long,Long,Long,Long,Long,Long,Long",
                                            "Long,Long,Long,Long,Long,Long,Long")
    val whereClausesByTable: Array[String] = Array(tables(0) + ".entity_id=" + inEntityId, tables(1) + ".entity_id=" + inEntityId,
                                                   tables(2) + ".entity_id=" + inEntityId, tables(3) + ".entity_id=" + inEntityId,
                                                   tables(4) + ".entity_id=" + inEntityId, tables(5) + ".entity_id=" + inEntityId,
                                                   tables(6) + ".entity_id=" + inEntityId)
    val orderByClausesByTable: Array[String] = Array("id", "id", "id", "id", "id", "entity_id", "group_id")

    // *******************************************
    //****** NOTE **********: some logic here for counting & looping has been commented out because it is not yet updated to work with the sorting of
    // attributes on an entity.  But it is left here because it was so carefully debugged, once, and seems likely to be used again if we want to limit the
    // data queried and sorted to that amount which can be displayed at a given time.  For example,
    // we could query first from the AttributeSorting table, then based on that decide for which ones to get all the data. But maybe for now there's a small
    // enough amount of data that we can query all rows all the time.
    // *******************************************

    // first just get a total row count for UI convenience later (to show how many left not viewed yet)
    // ABOUT THESE COMMENTED LINES: SEE "** NOTE **" ABOVE:
//    var totalRowsAvailable: Long = 0
//    var tableIndexForRowCounting = 0
//    while ((maxValsIn == 0 || totalRowsAvailable <= maxValsIn) && tableIndexForRowCounting < tables.length) {
//      val tableName = tables(tableIndexForRowCounting)
//      totalRowsAvailable += extractRowCountFromCountQuery("select count(*) from " + tableName + " where " + whereClausesByTable(tableIndexForRowCounting))
//      tableIndexForRowCounting += 1
//    }

    // idea: this could change to a val and be filled w/ a recursive helper method; other vars might go away then too.
    var tableListIndex: Int = 0

    // ABOUT THESE COMMENTED LINES: SEE "** NOTE **" ABOVE:
    //keeps track of where we are in getting rows >= inStartingObjectIndex and <= maxValsIn
    //    var counter: Long = 0
    //    while ((maxValsIn == 0 || counter - inStartingObjectIndex <= maxValsIn) && tableListIndex < tables.length) {
    while (tableListIndex < tables.length) {
      val tableName = tables(tableListIndex)
      // ABOUT THESE COMMENTED LINES: SEE "** NOTE **" ABOVE:
      //val thisTablesRowCount: Long = extractRowCountFromCountQuery("select count(*) from " + tableName + " where " + whereClausesByTable(tableListIndex))
      //if (thisTablesRowCount > 0 && counter + thisTablesRowCount >= inStartingObjectIndex) {
      //try {

          // Idea: could speed this query up in part? by doing on each query something like:
          //       limit maxValsIn+" offset "+ inStartingObjectIndex-counter;
          // ..and then incrementing the counters appropriately.
          // Idea: could do the sorting (currently done just before the end of this method) in sql? would have to combine all queries to all tables, though.
          val key = whereClausesByTable(tableListIndex).substring(0, whereClausesByTable(tableListIndex).indexOf("="))
          val columns = tableName + "." + columnsSelectedByTable(tableListIndex).replace(",", "," + tableName + ".")
          var sql: String = "select attributesorting.sorting_index, " + columns +
                            " from " +
                            // idea: is the RIGHT JOIN really needed, or can it be a normal join? ie, given tables' setup can there really be
                            // rows of any Attribute (or RelationTo*) table without a corresponding attributesorting row?  Going to assume not,
                            // for some changes below adding the sortingindex parameter to the Attribute constructors, for now at least until this is studied
                            // again.  Maybe it had to do with the earlier unreliability of always deleting rows from attributesorting when Attributes were
                            // deleted (and in fact an attributesorting can in theory still be created without an Attribute row, and maybe other such problems).
                            "   attributesorting RIGHT JOIN " + tableName +
                            "     ON (attributesorting.attribute_form_id=" + PostgreSQLDatabase.getAttributeFormId(tableName) +
                            "     and attributesorting.attribute_id=" + tableName + ".id )" +
                            "   JOIN entity ON entity.id=" + key +
                            " where " +
                            (if (!showAllArchivedEntities) {
                              "(not entity.archived) and "
                            } else {
                              ""
                            }) +
                            whereClausesByTable(tableListIndex)
          if (tableName.toLowerCase == Controller.RELATION_TO_ENTITY_TYPE && !showAllArchivedEntities) {
            sql += " and not exists(select 1 from entity e2, relationtoentity rte2 where e2.id=rte2.entity_id_2" +
                   " and relationtoentity.entity_id_2=rte2.entity_id_2 and e2.archived)"
          }
          sql += " order by " + tableName + "." + orderByClausesByTable(tableListIndex)
          val results = dbQuery(sql, typesByTable(tableListIndex))
          for (result: Array[Option[Any]] <- results) {
            // skip past those that are outside the range to retrieve
            //idea: use some better scala/function construct here so we don't keep looping after counter hits the max (and to make it cleaner)?
            //idea: move it to the same layer of code that has the Attribute classes?

            // ABOUT THESE COMMENTED LINES: SEE "** NOTE **" ABOVE:
            // Don't get it if it's not in the requested range:
//            if (counter >= inStartingObjectIndex && (maxValsIn == 0 || counter <= inStartingObjectIndex + maxValsIn)) {
              if (tableName == Controller.QUANTITY_TYPE) {
                allResults.add((if (result(0).isEmpty) None else Some(result(0).get.asInstanceOf[Long]),
                           new QuantityAttribute(this, result(1).get.asInstanceOf[Long], result(2).get.asInstanceOf[Long], result(3).get.asInstanceOf[Long],
                                                 result(4).get.asInstanceOf[Long], result(5).get.asInstanceOf[Float],
                                                 if (result(6).isEmpty) None else Some(result(6).get.asInstanceOf[Long]), result(7).get.asInstanceOf[Long],
                                                 result(0).get.asInstanceOf[Long])))
              } else if (tableName == Controller.TEXT_TYPE) {
                allResults.add((if (result(0).isEmpty) None else Some(result(0).get.asInstanceOf[Long]),
                           new TextAttribute(this, result(1).get.asInstanceOf[Long], result(2).get.asInstanceOf[Long], result(3).get.asInstanceOf[Long],
                                             result(4).get.asInstanceOf[String], if (result(5).isEmpty) None else Some(result(5).get.asInstanceOf[Long]),
                                             result(6).get.asInstanceOf[Long], result(0).get.asInstanceOf[Long])))
              } else if (tableName == Controller.DATE_TYPE) {
                allResults.add((if (result(0).isEmpty) None else Some(result(0).get.asInstanceOf[Long]),
                           new DateAttribute(this, result(1).get.asInstanceOf[Long], result(2).get.asInstanceOf[Long], result(3).get.asInstanceOf[Long],
                                             result(4).get.asInstanceOf[Long], result(0).get.asInstanceOf[Long])))
              } else if (tableName == Controller.BOOLEAN_TYPE) {
                allResults.add((if (result(0).isEmpty) None else Some(result(0).get.asInstanceOf[Long]),
                           new BooleanAttribute(this, result(1).get.asInstanceOf[Long], result(2).get.asInstanceOf[Long], result(3).get.asInstanceOf[Long],
                                                result(4).get.asInstanceOf[Boolean], if (result(5).isEmpty) None else Some(result(5).get.asInstanceOf[Long]),
                                                result(6).get.asInstanceOf[Long], result(0).get.asInstanceOf[Long])))
              } else if (tableName == Controller.FILE_TYPE) {
                allResults.add((if (result(0).isEmpty) None else Some(result(0).get.asInstanceOf[Long]),
                           new FileAttribute(this, result(1).get.asInstanceOf[Long], result(2).get.asInstanceOf[Long], result(3).get.asInstanceOf[Long],
                                             result(4).get.asInstanceOf[String], result(5).get.asInstanceOf[Long], result(6).get.asInstanceOf[Long],
                                             result(7).get.asInstanceOf[String], result(8).get.asInstanceOf[Boolean], result(9).get.asInstanceOf[Boolean],
                                             result(10).get.asInstanceOf[Boolean], result(11).get.asInstanceOf[Long], result(12).get.asInstanceOf[String],
                                             result(0).get.asInstanceOf[Long])))
              } else if (tableName == Controller.RELATION_TO_ENTITY_TYPE) {
                allResults.add((if (result(0).isEmpty) None else Some(result(0).get.asInstanceOf[Long]),
                           new RelationToEntity(this, result(1).get.asInstanceOf[Long], result(2).get.asInstanceOf[Long], result(3).get.asInstanceOf[Long],
                                                result(4).get.asInstanceOf[Long],
                                                if (result(5).isEmpty) None else Some(result(5).get.asInstanceOf[Long]), result(6).get.asInstanceOf[Long],
                                                result(0).get.asInstanceOf[Long])))
              } else if (tableName == Controller.RELATION_TO_GROUP_TYPE) {
                allResults.add((if (result(0).isEmpty) None else Some(result(0).get.asInstanceOf[Long]),
                           new RelationToGroup(this, result(1).get.asInstanceOf[Long], result(2).get.asInstanceOf[Long], result(3).get.asInstanceOf[Long],
                                               result(4).get.asInstanceOf[Long],
                                               if (result(5).isEmpty) None else Some(result(5).get.asInstanceOf[Long]),
                                               result(6).get.asInstanceOf[Long], result(0).get.asInstanceOf[Long])))
              } else throw new OmDatabaseException("invalid table type?: '" + tableName + "'")

            // ABOUT THESE COMMENTED LINES: SEE "** NOTE **" ABOVE:
            //}
//            counter += 1
          }

      // ABOUT THESE COMMENTED LINES: SEE "** NOTE **" ABOVE:
        //}
        //remove the try permanently, or, what should be here as a 'catch'? how interacts w/ 'throw' or anything related just above?
      //} else {
      //  counter += thisTablesRowCount
      //}
      tableListIndex += 1
    }

    val allResultsArray: Array[(Long, Attribute)] = new Array[(Long, Attribute)](allResults.size)
    var index = -1
    for (element: (Option[Long], Attribute) <- allResults.toArray(new Array[(Option[Long], Attribute)](0))) {
      index += 1
      // using maxIdValue as the max value of a long so those w/o sorting information will just sort last:
      allResultsArray(index) = (element._1.getOrElse(maxIdValue), element._2)
    }
    // Per the scaladocs for scala.math.Ordering, this sorts by the first element of the tuple (ie, .z_1) which at this point is attributesorting.sorting_index.
    // (The "getOrElse" on next line is to allow for the absence of a value in case the attributeSorting table doesn't have an entry for some attributes.
    Sorting.quickSort(allResultsArray)(Ordering[Long].on(x => x._1.asInstanceOf[Long]))

    val from: Int = inStartingObjectIndex
    val numVals: Int = if (maxValsIn > 0) maxValsIn else allResultsArray.length
    val until: Int = Math.min(inStartingObjectIndex + numVals, allResultsArray.length)
    (allResultsArray.slice(from, until), allResultsArray.length)
  }

  /** The 2nd parameter is to avoid saying an entity is a duplicate of itself: checks for all others only. */
  def isDuplicateEntity(inName: String, inSelfIdToIgnore: Option[Long] = None): Boolean = {
    val first = isDuplicateRow(inName, Controller.ENTITY_TYPE, "id", "name",
                               if (!showAllArchivedEntities) {
                                 Some("(not archived)")
                               } else {
                                 None
                               },
                               inSelfIdToIgnore)
    val second = isDuplicateRow(inName, Controller.RELATION_TYPE_TYPE, "entity_id", "name_in_reverse_direction", None, inSelfIdToIgnore)
    first || second
  }

  ///** The inSelfIdToIgnore parameter is to avoid saying a class is a duplicate of itself: checks for all others only. */
  def isDuplicateRow(possibleDuplicateIn: String, table: String, keyColumnToIgnoreOn: String, columnToCheckForDupValues: String, extraCondition: Option[String],
                     inSelfIdToIgnore: Option[Long] = None): Boolean = {
    val valueToCheck: String = escapeQuotesEtc(possibleDuplicateIn)

    val exception: String =
      if (inSelfIdToIgnore.isEmpty) ""
      else "and not " + keyColumnToIgnoreOn + "=" + inSelfIdToIgnore.get.toString

    doesThisExist("SELECT count(" + keyColumnToIgnoreOn + ") from " + table + " where " +
                  (if (extraCondition.isDefined && extraCondition.get.nonEmpty) extraCondition.get else "true") +
                  " and lower(" + columnToCheckForDupValues + ")=lower('" + valueToCheck + "') " + exception,
                  failIfMoreThanOneFoundIn = false)
  }


  /** The 2nd parameter is to avoid saying a class is a duplicate of itself: checks for all others only. */
  def isDuplicateClass(inName: String, inSelfIdToIgnore: Option[Long] = None): Boolean = {
    isDuplicateRow(inName, "class", "id", "name", None, inSelfIdToIgnore)
  }

  /**
   * Like jdbc's default, if you don't call begin/rollback/commit, it will commit after every stmt,
   * using the default behavior of jdbc; but if you call begin/rollback/commit, it will let you manage
   * explicitly and will automatically turn autocommit on/off as needed to allow that.
   */
  def beginTrans() {
    // implicitly begins a transaction, according to jdbc documentation
    mConn.setAutoCommit(false)
  }

  def rollbackTrans() {
    mConn.rollback()
    // so future work is auto- committed unless programmer explicitly opens another transaction
    mConn.setAutoCommit(true)
  }

  def commitTrans() {
    mConn.commit()
    // so future work is auto- committed unless programmer explicitly opens another transaction
    mConn.setAutoCommit(true)
  }

  def maxIdValue: Long = {
    // Max size for a Java long type, and for a postgresql 7.2.1 bigint type (which is being used, at the moment, for the id value in Entity table.
    // (these values are from file:///usr/share/doc/postgresql-doc-9.1/html/datatype-numeric.html)
    9223372036854775807L
  }

  def minIdValue: Long = {
    -9223372036854775808L
  }

  protected override def finalize() {
    super.finalize()
    if (mConn != null) mConn.close()
  }

  def extractRowCountFromCountQuery(inSQL: String): Long = {
    val results = dbQueryWrapperForOneRow(inSQL, "Long")
    // not checking for None here as its presence would be a bug:
    val result: Long = results(0).get.asInstanceOf[Long]
    result
  }

  /** Convenience function. Error message it gives if > 1 found assumes that sql passed in will return only 1 row! */
  def doesThisExist(inSql: String, failIfMoreThanOneFoundIn: Boolean = true): Boolean = {
    val rowcnt: Long = extractRowCountFromCountQuery(inSql)
    if (failIfMoreThanOneFoundIn) {
      if (rowcnt == 1) true
      else if (rowcnt > 1) throw new OmDatabaseException("Should there be > 1 entries for sql: " + inSql + "?? (" + rowcnt + " were found.)")
      else false
    }
    else rowcnt >= 1
  }

  /** Cloned to archiveObjects: CONSIDER UPDATING BOTH if updating one.  Returns the # of rows deleted.
    * Unless the parameter rowsExpected==-1, it will allow any # of rows to be deleted; otherwise if the # of rows is wrong it will abort tran & fail.
    */
  private def deleteObjects(tableNameIn: String, whereClauseIn: String, rowsExpected: Long = 1, callerManagesTransactions: Boolean = false): Long = {
    //idea: enhance this to also check & return the # of rows deleted, to the caller to just make sure? If so would have to let caller handle transactions.
    val sql = "DELETE FROM " + tableNameIn + " " + whereClauseIn
    if (!callerManagesTransactions) beginTrans()
    try {
      val rowsDeleted = dbAction(sql, callerChecksRowCountEtc = true)
      if (rowsExpected >= 0 && rowsDeleted != rowsExpected) {
        // Roll back, as we definitely don't want to delete an unexpected # of rows.
        // Do it ***EVEN THOUGH callerManagesTransaction IS true***: seems cleaner/safer this way.
        throw rollbackWithCatch(new OmDatabaseException("Delete command would have removed " + rowsDeleted + " rows, but " +
                                              rowsExpected + " were expected! Did not perform delete.  SQL is: \"" + sql + "\""))
      } else {
        if (!callerManagesTransactions) commitTrans()
        rowsDeleted
      }
    } catch {
      case e: Exception => throw rollbackWithCatch(e)
    }
  }

  /** Cloned from deleteObjects: CONSIDER UPDATING BOTH if updating one.
    */
  private def archiveObjects(tableNameIn: String, whereClauseIn: String, rowsExpected: Long = 1, callerManagesTransactions: Boolean = false,
                             unarchive: Boolean = false) {
    //idea: enhance this to also check & return the # of rows deleted, to the caller to just make sure? If so would have to let caller handle transactions.
    if (!callerManagesTransactions) beginTrans()
    try {
      val archive = if (unarchive) "false" else "true"
      val archivedDate = if (unarchive) {
        "NULL"
      } else {
        "" + System.currentTimeMillis()
      }
      val rowsAffected = dbAction("update " + tableNameIn + " set (archived, archived_date) = (" + archive + ", " + archivedDate + ") " + whereClauseIn)
      if (rowsExpected >= 0 && rowsAffected != rowsExpected) {
        // Roll back, as we definitely don't want to affect an unexpected # of rows.
        // Do it ***EVEN THOUGH callerManagesTransaction IS true***: seems cleaner/safer this way.
        throw rollbackWithCatch(new OmDatabaseException("Archive command would have updated " + rowsAffected + "rows, but " +
                                              rowsExpected + " were expected! Did not perform archive."))
      } else {
        if (!callerManagesTransactions) commitTrans()
      }
    } catch {
      case e: Exception => throw rollbackWithCatch(e)
    }
  }

  private def deleteObjectById(inTableName: String, inID: Long, callerManagesTransactions: Boolean = false): Unit = {
    deleteObjects(inTableName, "where id=" + inID, callerManagesTransactions = callerManagesTransactions)
  }

  /**
   * Although the next sequence value would be set automatically as the default for a column (at least the
   * way I have them defined so far in postgresql); we do it explicitly
   * so we know what sequence value to return, and what the unique key is of the row we just created!
   */
  private def getNewKey(inSequenceName: String): /*id*/ Long = {
    val result: Long = dbQueryWrapperForOneRow("SELECT nextval('" + inSequenceName + "')", "Long")(0).get.asInstanceOf[Long]
    result
  }

  // (idea: find out: why doesn't compiler (ide or cli) complain when the 'override' is removed from next line?)
  // idea: see comment on findUnusedSortingIndex
  def findIdWhichIsNotKeyOfAnyEntity: Long = {
    //better idea?  This should be fast because we start in remote regions and return as soon as an unused id is found, probably
    //only one iteration, ever.  (See similar comments elsewhere.)
    val startingId: Long = maxIdValue - 1

    @tailrec def findIdWhichIsNotKeyOfAnyEntity_helper(workingId: Long, counter: Long): Long = {
      if (entityKeyExists(workingId)) {
        if (workingId == maxIdValue) {
          // means we did a full loop across all possible ids!?  Doubtful. Probably would turn into a performance problem long before. It's a bug.
          throw new OmDatabaseException("No id found which is not a key of any entity in the system. How could all id's be used??")
        }
        // idea: this check assumes that the thing to get IDs will re-use deleted ones and wrap around the set of #'s. That fix is on the list (informally
        // at this writing, 2013-11-18).
        if (counter > 1000) throw new OmDatabaseException("Very unexpected, but could it be that you are running out of available entity IDs?? Have someone check, " +
                                                "before you need to create, for example, a thousand more entities.")
        findIdWhichIsNotKeyOfAnyEntity_helper(workingId - 1, counter + 1)
      } else workingId
    }

    findIdWhichIsNotKeyOfAnyEntity_helper(startingId, 0)
  }

  // (see note in ImportExport's call to this, on this being better in the class and action *tables*, but here for now until those features are ready)
  def addUriEntityWithUriAttribute(containingEntityIn: Entity, newEntityNameIn: String, uriIn: String, observationDateIn: Long,
                                   makeThemPublicIn: Option[Boolean], callerManagesTransactionsIn: Boolean,
                                   quoteIn: Option[String] = None): (Entity, RelationToEntity) = {
    if (quoteIn.isDefined) require(!quoteIn.get.isEmpty, "It doesn't make sense to store a blank quotation; there was probably a program error.")
    if (!callerManagesTransactionsIn) beginTrans()
    try {
      // **idea: BAD SMELL: should this method be moved out of the db class, since it depends on higher-layer components, like EntityClass and
      // those in the same package? It was in Controller, but moved here
      // because it seemed like things that manage transactions should be in the db layer.  So maybe it needs un-mixing of layers.

      val (uriClassId: Long, uriClassTemplateId: Long) = getOrCreateClassAndTemplateEntityIds("URI", callerManagesTransactionsIn)
      val (_, quotationClassTemplateId: Long) = getOrCreateClassAndTemplateEntityIds("quote", callerManagesTransactionsIn)
      val (newEntity: Entity, newRTE: RelationToEntity) = containingEntityIn.createEntityAndAddHASRelationToIt(newEntityNameIn, observationDateIn,
                                                                                                               makeThemPublicIn, callerManagesTransactionsIn)
      updateEntitysClass(newEntity.getId, Some(uriClassId), callerManagesTransactionsIn)
      newEntity.addTextAttribute(uriClassTemplateId, uriIn, None, None, observationDateIn, callerManagesTransactionsIn)
      if (quoteIn.isDefined) {
        newEntity.addTextAttribute(quotationClassTemplateId, quoteIn.get, None, None, observationDateIn, callerManagesTransactionsIn)
      }
      if (!callerManagesTransactionsIn) commitTrans()
      (newEntity, newRTE)
    } catch {
      case e: Exception =>
        if (!callerManagesTransactionsIn) rollbackTrans()
        throw e
    }
  }

  def getOrCreateClassAndTemplateEntityIds(classNameIn: String, callerManagesTransactionsIn: Boolean): (Long, Long) = {
    //(see note above re 'bad smell' in method addUriEntityWithUriAttribute.)
    if (!callerManagesTransactionsIn) beginTrans()
    try {
      val (classId, entityId) = {
        val foundId = findFIRSTClassIdByName(classNameIn, caseSensitive = true)
        if (foundId.isDefined) {
          val entityId: Long = new EntityClass(this, foundId.get).getTemplateEntityId
          (foundId.get, entityId)
        } else {
          val (classId: Long, entityId: Long) = createClassAndItsTemplateEntity(classNameIn)
          (classId, entityId)
        }
      }
      if (!callerManagesTransactionsIn) commitTrans()
      (classId, entityId)
    }
    catch {
      case e: Exception =>
        if (!callerManagesTransactionsIn) rollbackTrans()
        throw e
    }
  }

  def showAllArchivedEntities: Boolean = mShowAllArchivedEntities

  def setShowAllArchivedEntities(in: Boolean): Unit = {
    mShowAllArchivedEntities = in
  }

}