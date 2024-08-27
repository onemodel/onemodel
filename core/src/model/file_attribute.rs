/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
// use crate::model::attribute_with_valid_and_observed_dates::AttributeWithValidAndObservedDates;
use crate::model::database::DataType;
use crate::model::database::Database;
use crate::util::Util;
use anyhow::{anyhow, Error, Result};
// use sqlx::{PgPool, Postgres, Row, Transaction};
use crate::model::attribute::Attribute;
use crate::model::entity::Entity;
// use crate::model::id_wrapper::IdWrapper;
use crate::model::relation_type::RelationType;
use md5::{Digest, Md5};
use sqlx::{Postgres, Transaction};
use std::ffi::OsStr;
use std::path::Path;
use std::cell::{RefCell};
use std::rc::Rc;

// ***NOTE***: Similar/identical code found in *_attribute.rs, relation_to_entity.rs and relation_to_group.rs,
// due to Rust limitations on OO.  Maintain them all similarly.

/// See BooleanAttribute, TextAttribute etc for some comments.
pub struct FileAttribute<'a> {
    // For descriptions of the meanings of these variables, see the comments
    // on create_file_attribute(...) or create_tables() in PostgreSQLDatabase or Database structs,
    // and/or examples in the database testing code.
    id: i64,
    db: &'a dyn Database,
    already_read_data: bool,    /*= false*/
    parent_id: i64,             /*= 0_i64*/
    attr_type_id: i64,          /*= 0_i64*/
    sorting_index: i64,         /*= 0_i64*/
    description: String,        /*= null;*/
    original_file_date: i64,    /*= 0;*/
    stored_date: i64,           /*= 0;*/
    original_file_path: String, /*= null;*/
    readable: bool,             /*= false;*/
    writable: bool,             /*= false;*/
    executable: bool,           /*= false;*/
    size: i64,                  /*= 0;*/
    md5hash: String,            /*= null;*/
}

impl FileAttribute<'_> {
    /// This one is perhaps only called by the database class implementation (and a test)--so it
    /// can return arrays of objects & save more DB hits
    /// that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    /// one that already exists.  It does not confirm that the id exists in the db.
    fn new<'a>(
        db: &'a dyn Database,
        id: i64,
        parent_id: i64,
        attr_type_id: i64,
        description: String,
        original_file_date: i64,
        stored_date: i64,
        original_file_path: String,
        readable: bool,
        writable: bool,
        executable: bool,
        size: i64,
        md5hash: String,
        sorting_index: i64,
    ) -> FileAttribute<'a> {
        // idea: make the parameter order uniform throughout the system
        FileAttribute {
            id,
            db,
            already_read_data: true,
            parent_id,
            attr_type_id,
            description,
            original_file_date,
            stored_date,
            original_file_path,
            readable,
            writable,
            executable,
            size,
            md5hash,
            sorting_index,
        }
    }

    /// This constructor instantiates an existing object from the DB. You can use Entity.add*Attribute() to
    /// create a new object.
    pub fn new2<'a>(
        db: &'a dyn Database,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id: i64,
    ) -> Result<FileAttribute<'a>, anyhow::Error> {
        // (See comment in similar spot in BooleanAttribute for why not checking for exists, if db.is_remote.)
        if !db.is_remote() && !db.file_attribute_key_exists(transaction, id)? {
            Err(anyhow!("Key {}{}", id, Util::DOES_NOT_EXIST))
        } else {
            Ok(FileAttribute {
                id,
                db,
                already_read_data: false,
                parent_id: 0,
                attr_type_id: 0,
                sorting_index: 0,
                description: "".to_string(),
                original_file_date: 0,
                stored_date: 0,
                original_file_path: "".to_string(),
                readable: false,
                writable: false,
                executable: false,
                size: 0,
                md5hash: "".to_string(),
            })
        }
    }

    //Tested with Entity code.  Search for usages to see.
    fn md5_hash(path_in: &std::path::Path) -> Result<String, anyhow::Error> {
        // Best info/examples seem to be in the the github.com/RustCrypto/hashes
        // README file!

        // (Could see wkp re TOCTOU though, for risks and possible mitigations.  Conclusion: do nothing
        // about it for now).

        // Idea: combine somehow w/ similar logic in PostgreSQLDatabase.verifyFileAttributeContent ?

        //Yes, md5 hashes are considered obsolete.  They were not, when this was originally implemented in
        //scala, and as of this writing (2023-10) the goal is just to move to rust, and changing
        //the hash code (and the hashes on my existing files stored in OM) will have to wait.

        // First stab at this, and notes, based on crate docs and below-noted README, before I read farther:
        let mut hasher = Md5::new();
        // hasher.update(b"hello world");
        // acquire hash digest in the form of GenericArray,
        // which in this case is equivalent to [u8; 16]
        // let result = hasher.finalize();
        // (Next 3 lines are example code from the md-5 crate docs or the github.com/RustCrypto/hashes
        // README file, I think.
        // %%Note: SEE THAT FILE, which says 'update' can be called repeatedly, so maybe this could be redone
        // to read the file in chunks instead of all at once, calling hasher.update on each chunk,
        // if a test shows that give an identical result.)
        // hasher.update(b"hello world");
        // use hex_literal::hex;
        // assert_eq!(result[..], hex_literal::hex!("5eb63bbbe01eeed093cb22bb8f5acdc3"));
        // if ! path_in.try_exists()? {
        // }

        let mut file = std::fs::File::open(&path_in)?;
        let _n = std::io::copy(&mut file, &mut hasher)?;
        let _hash = hasher.finalize();
        let _dst = [0 as u8; 32];
        //%%later: low confidence the next line is right.  Doesn't compile (no "into_bytes" &c).  Ck tests.  See those docs?  What is dst for?
        // let hex_hash = base16ct::lower::encode_str(&hash.into_bytes(), &mut dst)?;
        let hex_hash = "%%fix above line and delete this one";
        return Ok(hex_hash.to_string());

        /*%%later: old scala code for this:
        let mut fis: java.io.FileInputStream = null;
        let d = java.security.MessageDigest.getInstance("MD5");
        try {
            fis = new java.io.FileInputStream(path_in)
            let buffer = new Array[Byte](2048);
            let mut numBytesRead = 0;
            @tailrec
            fn calculateRest() {
                //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                numBytesRead = fis.read(buffer)
                //noinspection RemoveRedundantReturn  //left intentionally for reading clarify
                if numBytesRead == -1) return
                else {
                    d.update(buffer, 0, numBytesRead)
                    calculateRest()
                }
            }
            calculateRest()
        }
        finally if fis != null) fis.close()
        //%%
        //a handy value for testing code like above, in comparison with the md5sum command on a file containing only "1234" (w/o quotes) and a linefeed (size 5):
        // let ba = Array[Byte]('1', '2', '3', '4', 10);
        //so then in scala REPL (interpreter) you set "val d..." as above, "d.update(ba)", and the below:
        // outputs same as command 'md5sum <file>':
        let md5sum: String = {
            //noinspection LanguageFeature  // (the '&' use on next line is an 'advanced feature' style violation but it's the way i found to do it ...)
            d.digest.map(0xFF &).map {"%02x".format(_)}.foldLeft("") {_ + _}
        }
        md5sum
         */
    }

    fn get_filename_filler() -> String {
        "aaa".to_string()
    }

    /// Returns a prefix and suffix (like, "filename" and ".ext").
    /// I.e., was for use with java.nio.file.Files.createTempFile (which makes sure it will not collide with existing names).
    /// Calling this likely presumes that the caller has already decided not to use the old path, or at least the old filename in the temp directory.
    fn get_usable_filename(
        original_file_path_in: &str,
    ) -> Result<(String, String), anyhow::Error> {
        let file_name: &OsStr = match Path::new(original_file_path_in).file_name() {
            Some(s) => s,
            None => return Err(anyhow!("No file name in {} ?", original_file_path_in)),
        };
        let file_stem: String = match Path::new(file_name).file_stem() {
            Some(s) => s.to_string_lossy().into_owned(),
            None => {
                return Err(anyhow!(
                    "No file stem in the filename part of {} ?",
                    original_file_path_in
                ))
            }
        };

        // base_name (in scala anyway) had to be at least 3 chars, for createTempFile:
        let base_name_padded: String =
            format!("{}{}", file_stem, FileAttribute::get_filename_filler());
        let length_to_take = std::cmp::max(file_stem.len(), 3);
        let base_name = Util::substring_from_start(base_name_padded.as_str(), length_to_take);

        let dot_and_extension: String = {
            let dotless_extension = match Path::new(file_name).extension() {
                Some(s) => s.to_string_lossy().into_owned(),
                None => "".to_string(),
            };
            if dotless_extension.len() > 0 {
                format!(".{}", dotless_extension)
            } else {
                "".to_string()
            }
        };
        //for hidden files to stay that way unless the size is too small:
        if base_name == FileAttribute::get_filename_filler() && dot_and_extension.len() >= 3 {
            Ok((dot_and_extension, "".to_string()))
        } else {
            Ok((base_name, dot_and_extension))
        }
    }

    fn get_dates_description(&mut self) -> Result<String, anyhow::Error> {
        Ok(format!(
            "mod {}, stored {}",
            Util::useful_date_format(self.get_original_file_date()?),
            Util::useful_date_format(self.get_stored_date()?)
        ))
    }

    fn get_original_file_date(&mut self) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(None)?;
        }
        Ok(self.original_file_date)
    }

    fn get_stored_date(&mut self) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(None)?;
        }
        Ok(self.stored_date)
    }

    fn get_description(&mut self) -> Result<String, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(None)?;
        }
        Ok(self.description.clone())
    }

    fn get_original_file_path(&mut self) -> Result<String, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(None)?;
        }
        Ok(self.original_file_path.clone())
    }

    fn get_size(&mut self) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(None)?;
        }
        Ok(self.size)
    }

    fn get_md5hash(&mut self) -> Result<String, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(None)?;
        }
        Ok(self.md5hash.clone())
    }

    fn get_readable(&mut self) -> Result<bool, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(None)?;
        }
        Ok(self.readable)
    }

    fn get_writeable(&mut self) -> Result<bool, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(None)?;
        }
        Ok(self.writable)
    }

    fn get_executable(&mut self) -> Result<bool, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(None)?;
        }
        Ok(self.executable)
    }

    /// just calling the File.get_usable_space function on a nonexistent file yields 0, so come up with something better. -1 if it just can't figure it out.
    fn get_usable_space(_file_in: &Path) -> i64 {
        //%%to impl this, see the comparisons in
        //  https://github.com/heim-rs/heim/blob/master/COMPARISON.md
        // ...and also possibly the crates: sys_metrics, sysinfo-report, sc-sysinfo,
        // try {
        //     if file_in.exists) file_in.get_usable_space()
        //     else if file_in.getParentFile == null) -1
        //     else get_usable_space(file_in.getParentFile)
        // } catch {
        //     // oh well, give up.
        //     case e: Exception => -1
        // }
        -1
    }

    /* %%impl these after more basics are done

        // Idea: how make the 2nd parameter an option with None as default, instead of null as default?
        fn retrieveContent(file_in: File, damageFileForTesting: (File) => Unit = null) {
            let mut outputStream: FileOutputStream = null;
            try {
                if (!file_in.exists()) || file_in.length() < this.get_size()) {
                    let space = get_usable_space(file_in);
                    if space > -1 && space < this.get_size()) throw new OmException("Not enough space on disk to retrieve file of size " + this.get_file_size_description + ".")
                }
                outputStream = new FileOutputStream(file_in)
                //idea: if the file exists, copy out to a temp name, then after retrieval delete it & rename the new one to it? (uses more space)
                let (sizeStoredInDb, hashStoredInDb) = db.get_file_attribute_content(get_id, outputStream);
                // idea: this could be made more efficient if we checked the hash during streaming it to the local disk (in db.get_file_attribute_content)
                // (as does db.verifyFileAttributeContent).

                // this is a hook so tests can verify that we do fail if the file isn't intact
                // (Idea:  huh?? This next line does nothing. Noted in tasks to see what is meant & make it do that. or at least more clear.)
                //noinspection ScalaUselessExpression  //left intentionally for reading clarify
                if damageFileForTesting != null) damageFileForTesting

                let downloadedFilesHash = FileAttribute::md5_hash(file_in);
                if file_in.length != sizeStoredInDb) throw new OmFileTransferException("File sizes differ!: stored/downloaded: " + sizeStoredInDb + " / " + file_in
                    .length())
                if downloadedFilesHash != hashStoredInDb) throw new OmFileTransferException("The md5sum hashes differ!: stored/downloaded: " + hashStoredInDb + " / "
                    + downloadedFilesHash)
                file_in.setReadable(self.get_readable())
                file_in.setWritable(self.get_writeable())
                file_in.setExecutable(self.get_executable())
            } finally {
                if outputStream != null) outputStream.close()
            }
        }
    */
    fn get_permissions_description(&mut self) -> Result<String, anyhow::Error> {
        //ex: rwx or rw-, like "ls -l" does
        Ok(format!(
            "{}{}{}",
            if self.get_readable()? { "r" } else { "-" },
            if self.get_writeable()? { "w" } else { "-" },
            if self.get_executable()? { "x" } else { "-" }
        ))
    }

    fn get_file_size_description(&mut self) -> String {
        /*%%
           // note: it seems that (as per SI? IEC?), 1024 bytes is now 1 "binary kilobyte" aka kibibyte or KiB, etc.
           let decimalFormat = new java.text.DecimalFormat("0");
           if get_size() < math.pow(10, 3)) "" + get_size() + " bytes"
           else if get_size() < math.pow(10, 6)) "" + decimalFormat.format(get_size() / math.pow(10, 3)) + "kB (" + get_size() + ")"
           else if get_size() < math.pow(10, 9)) "" + decimalFormat.format(get_size() / math.pow(10, 6)) + "MB (" + get_size() + ")"
           else "" + decimalFormat.format(get_size() / math.pow(10, 9)) + "GB (" + get_size() + ")"
        */
        "[%%fill in fn get_file_size_description]".to_string()
    }

    /*
    fn update(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        attr_type_id_in: i64,
        boolean_in: bool,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<(), anyhow::Error> {
        // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
        // it all goes with
        self.db.update_boolean_attribute(
            transaction,
            self.id,
            self.get_parent_id(transaction)?,
            attr_type_id_in,
            boolean_in,
            valid_on_date_in,
            observation_date_in,
        )?;
        self.boolean_value = boolean_in;
        // (next line is already set by just-above call to get_parent_id().)
        // self.already_read_data = true;
        self.attr_type_id = attr_type_id_in;
        self.valid_on_date = valid_on_date_in;
        self.observation_date = observation_date_in;
        Ok(())
    }

    %%%see BOTH below update()s? dift sigs, so an update2()? sch for uses.

    // We don't update the dates, path, size, hash because we set those based on the file's own timestamp, path current date,
    // & contents when it is *first* written. So the only point to having an update method might be the attribute type & description.
    // AND note that: The dates for a fileAttribute shouldn't ever be None/NULL like with other Attributes, because it is the file date in the filesystem
    // before it was
    // read into OM, and the current date; so they should be known whenever adding a document.
    fn update(attr_type_id_in: Option<i64> = None, description_in: Option<String> = None) {
        // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
        // it all goes with
        let descr = if description_in.is_some()) description_in.get else get_description();
        let attr_type_id = if attr_type_id_in.is_some()) attr_type_id_in.get else get_attr_type_id();
        db.update_file_attribute(get_id, get_parent_id(), attr_type_id, descr)
        description = descr
        attr_type_id = attr_type_id
    }

    /// Using Options for the parameters so caller can pass in only those desired (named), and other members will stay the same.
    //fn update(attr_type_id_in: Option<i64> = None, description_in: Option<String> = None, original_file_date_in: Option<i64> = None,
    //           stored_date_in: Option<i64> = None, original_file_path_in: Option<String> = None, size_in: Option<i64> = None, md5_hash_in: Option<String> = None) {
    //  // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
    //  // it all goes with
    //  // ********IF THIS METHOD IS EVER UNCOMMENTED: BE SURE TO TEST THAT the values (like size, hash, original date,
    // stored date!) are untouched if unchanged if
    //  // not passed in!! And probably need to add the 3 boolean fields to it & test.
    //  db.update_file_attribute(id, parent_id,
    //                          if attr_type_id_in == None) get_attr_type_id() else in_attr_type_id.get,
    //                          if description_in == None) get_description() else in_description.get,
    //                          if original_file_date_in == None) get_original_file_date() else original_file_date_in.get,
    //                          if stored_date_in == None) get_stored_date() else stored_date_in.get,
    //                          if original_file_path_in == None) get_original_file_path() else original_file_path_in.get,
    //                          if size_in == None) get_size() else size_in.get,
    //                          if md5_hash_in == None) getMd5hash else md5_hash_in.get)
    //}

    */
}

impl Attribute for FileAttribute<'_> {
    /// Return some string. See comments on QuantityAttribute.get_display_string regarding the parameters.
    fn get_display_string(
        &mut self,
        length_limit_in: usize,
        _unused: Option<Entity>,        /*= None*/
        _unused2: Option<RelationType>, /*=None*/
        simplify: bool,                 /* = false*/
    ) -> Result<String, anyhow::Error> {
        let attr_type_id = self.get_attr_type_id(None)?;
        let type_name: String = match self.db.get_entity_name(None, attr_type_id)? {
            None => "(None)".to_string(),
            Some(x) => x,
        };
        let mut result: String = format!(
            "{} ({}); {}",
            self.get_description()?,
            type_name,
            self.get_file_size_description()
        );
        if !simplify {
            result = format!(
                "{} {} from {}, {}; md5 {}.",
                result,
                self.get_permissions_description()?,
                self.get_original_file_path()?,
                self.get_dates_description()?,
                self.get_md5hash()?
            );
        }
        Ok(Util::limit_attribute_description_length(
            result.as_str(),
            length_limit_in,
        ))
    }
    //old/deletable after above tested?:
    // fn get_display_string(length_limit_in: Int, unused: Option<Entity> = None, unused2: Option[RelationType] = None, simplify: bool = false) -> String {
    //     let type_name: String = db.get_entity_name(get_attr_type_id()).get;
    //     let mut result: String = get_description() + " (" + type_name + "); " + get_file_size_description;
    //     if ! simplify result = result + " " + get_permissions_description() + " from " +
    //                              get_original_file_path() + ", " + get_dates_description + "; md5 " + self.get_md5hash() + "."
    //     Attribute.limit_attribute_description_length(result, length_limit_in)
    // }

    fn read_data_from_db(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<(), anyhow::Error> {
        let data: Vec<Option<DataType>> = self.db.get_file_attribute_data(transaction, self.id)?;
        if data.len() == 0 {
            return Err(anyhow!(
                "No results returned from data request for: {}",
                self.id
            ));
        }

        self.already_read_data = true;
        self.description = match data[1].clone() {
            Some(DataType::String(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[1])),
        };
        self.original_file_date = match data[4] {
            Some(DataType::Bigint(b)) => b,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[4])),
        };
        self.stored_date = match data[5] {
            Some(DataType::Bigint(b)) => b,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[5])),
        };
        self.original_file_path = match data[6].clone() {
            Some(DataType::String(b)) => b,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[6])),
        };
        self.readable = match data[7] {
            Some(DataType::Boolean(b)) => b,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[7])),
        };
        self.writable = match data[8] {
            Some(DataType::Boolean(b)) => b,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[8])),
        };
        self.executable = match data[9] {
            Some(DataType::Boolean(b)) => b,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[9])),
        };
        self.size = match data[10] {
            Some(DataType::Bigint(b)) => b,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[10])),
        };
        self.md5hash = match data[11].clone() {
            Some(DataType::String(b)) => b,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[11])),
        };

        //BEGIN COPIED BLOCK descended from Attribute.assign_common_vars (unclear how to do better for now):
        self.parent_id = match data[0] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[0])),
        };
        self.attr_type_id = match data[2] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[2])),
        };
        // DataType::Bigint(self.sorting_index) = data[5];
        self.sorting_index = match data[3] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[3])),
        };
        //END COPIED BLOCK descended from Attribute.assign_common_vars (might be in comment in boolean_attribute.rs)

        Ok(())
    }
    // protected fn read_data_from_db() {
    //   description = faTypeData(1).get.asInstanceOf[String]
    //   original_file_date = faTypeData(4).get.asInstanceOf[i64]
    //   stored_date = faTypeData(5).get.asInstanceOf[i64]
    //   original_file_path = faTypeData(6).get.asInstanceOf[String]
    //   mReadable = faTypeData(7).get.asInstanceOf[bool]
    //   mWritable = faTypeData(8).get.asInstanceOf[bool]
    //   mExecutable = faTypeData(9).get.asInstanceOf[bool]
    //   mSize = faTypeData(10).get.asInstanceOf[i64]
    //   mMd5hash = faTypeData(11).get.asInstanceOf[String]
    // assign_common_vars(faTypeData(0).get.asInstanceOf[i64], faTypeData(2).get.asInstanceOf[i64], faTypeData(3).get.asInstanceOf[i64])
    // }

    /** Removes this object from the system. */
    fn delete<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        //id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.db.delete_file_attribute(transaction, self.id)
    }

    // This datum is provided upon construction (new2(), at minimum), so can be returned
    // regardless of already_read_data / read_data_from_db().
    fn get_id(&self) -> i64 {
        self.id
    }

    fn get_form_id(&self) -> Result<i32, Error> {
        // self.db.get_attribute_form_id(was in scala:  this.getClass.getSimpleName)
        //%% Since not using the reflection(?) from the line above, why not just return a constant
        //here?  What other places call the below method and its reverse? Do they matter now?
        self.db.get_attribute_form_id(Util::BOOLEAN_TYPE)
    }

    fn get_attr_type_id(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.attr_type_id)
    }

    fn get_sorting_index(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.sorting_index)
    }

    fn get_parent_id(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.parent_id)
    }
}

#[cfg(test)]
mod test {
    /*%%put this back after similar place in boolean_attribute.rs is resolved and this can be similarly:
     "get_display_string" should "return correct string and length" in {
       let mock_db = mock[PostgreSQLDatabase];
       let entity_id: i64 = 0;
       let other_entity_id: i64 = 1;
       let fileAttributeId: i64 = 0;
       //arbitrary, in milliseconds:
       let modifiedDate: i64 = 304;
       let stored_date: i64 = modifiedDate + 1;
       let attr_type_name = "txt format";
       let longDescription = "this is a longish description of a file";
       let filePath = "/tmp/w.jpeg";
       let size = 12345678;
       //noinspection SpellCheckingInspection
       let hash = "e156b9a37060ccbcbffe5ec0fc967016";
       when(mock_db.get_entity_name(other_entity_id)).thenReturn(Some(attr_type_name))
       when(mock_db.file_attribute_key_exists(fileAttributeId)).thenReturn(true)

       // (using arbitrary numbers for the unnamed parameters):
       let fileAttribute = new FileAttribute(mock_db, fileAttributeId, entity_id, other_entity_id, longDescription, modifiedDate, stored_date, filePath, true, true,;
                                             false, size, hash, 0)
       let small_limit = 35;
       let display1: String = fileAttribute.get_display_string(small_limit);
       let whole_thing: String = longDescription + " (" + attr_type_name + "); 12MB (12345678) rw- from " + filePath + ", " +;
                                "mod Wed 1969-12-31 17:00:00:" + modifiedDate + " MST, " +
                                "stored Wed 1969-12-31 17:00:00:" + stored_date + " MST; md5 " +
                                hash + "."
       let expected: String = whole_thing.substring(0, small_limit - 3) + "..." // put the real string here instead of dup logic?;
       assert(display1 == expected)

       let unlimited = 0;
       let display2: String = fileAttribute.get_display_string(unlimited);
       assert(display2 == whole_thing)
     }

     "getReplacementFilename" should "work" in {
       // This all is intended to make the file names made by File.createTempFile come out in a nice (readable/memorable/similar such as for hidden) way when it
       // is used.
       let fileAttributeId = 0L;
       let mock_db = mock[PostgreSQLDatabase];
       when(mock_db.file_attribute_key_exists(fileAttributeId)).thenReturn(true)

       let mut originalName = "";
       let fa: FileAttribute = new FileAttribute(mock_db, fileAttributeId) {override fn get_original_file_path() -> String { originalName}};
       let (basename, extension) = FileAttribute.get_usable_filename(fa.get_original_file_path());
       assert(basename == FileAttribute::get_filename_filler() && extension == "")

       originalName = "something.txt"
       let (basename2, extension2) = FileAttribute.get_usable_filename(fa.get_original_file_path());
       assert(basename2 == "something" && extension2 == ".txt")

       originalName = "someFilename"
       let (basename3, extension3) = FileAttribute.get_usable_filename(fa.get_original_file_path());
       assert(basename3 == "someFilename" && extension3 == "")

       originalName = ".hidden"
       let (basename4, extension4) = FileAttribute.get_usable_filename(fa.get_original_file_path());
       assert(basename4 == ".hidden" && extension4 == "")

       originalName = "1.txt"
       let (basename5, extension5) = FileAttribute.get_usable_filename(fa.get_original_file_path());
       assert(basename5 == "1aa" && extension5 == ".txt")

       originalName = "some.long.thing"
       let (basename6, extension6) = FileAttribute.get_usable_filename(fa.get_original_file_path());
       assert(basename6 == "some.long" && extension6 == ".thing")
     }
    */
}
