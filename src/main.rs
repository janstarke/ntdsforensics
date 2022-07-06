use std::{path::Path, collections::HashMap};

use clap::Parser;
use libesedb::{EseDb, ColumnVariant, Value, Record};
use maplit::{hashmap, hashset};
use simplelog::{SimpleLogger, Config, TermLogger};
use anyhow::{Result, anyhow};

use crate::{dbrecord::DbRecord, person::Person};

mod dbrecord;
mod person;

#[derive(Parser)]
#[clap(name="ntds", author, version, about, long_about = None)]
struct Args {
    /// name of the file to analyze
    pub (crate) ntds_file: String,

    #[clap(flatten)]
    pub (crate) verbose: clap_verbosity_flag::Verbosity,
}

struct ColumnInformation {
    id: i32,
    name: String,
    variant: ColumnVariant,
}

impl ColumnInformation {
    pub fn new(id: i32, name: String, variant: ColumnVariant) -> Self {
        Self {
            id,
            name,
            variant,
        }
    }
}

const DS_RECORD_ID_INDEX_NAME: &str = "DNT_col";
const DS_PARENT_RECORD_ID_INDEX_NAME: &str = "PDNT_col";
const DS_RECORD_TIME_INDEX_NAME: &str = "time_col";
const DS_ANCESTORS_INDEX_NAME: &str = "Ancestors_col";
const DS_OBJECT_TYPE_ID_INDEX_NAME: &str = "ATTb590606";
const DS_OBJECT_NAME_INDEX_NAME: &str = "ATTm3";
const DS_OBJECT_NAME2_INDEX_NAME: &str = "ATTm589825";
const DS_OBJECT_GUIDINDEX_NAME: &str = "ATTk589826";
const DS_WHEN_CREATED_INDEX_NAME: &str = "ATTl131074";
const DS_WHEN_CHANGED_INDEX_NAME: &str = "ATTl131075";
const DS_USNCREATED_INDEX_NAME: &str = "ATTq131091";
const DS_USNCHANGED_INDEX_NAME: &str = "ATTq131192";
const DS_OBJECT_COL_INDEX_NAME: &str = "OBJ_col";
const DS_IS_DELETED_INDEX_NAME: &str = "ATTi131120";

const DS_ORIG_CONTAINER_ID_INDEX_NAME: &str = "ATTb590605";

const DS_SIDINDEX_NAME: &str = "ATTr589970";
const DS_SAMACCOUNT_NAME_INDEX_NAME: &str = "ATTm590045";
const DS_USER_PRINCIPAL_NAME_INDEX_NAME: &str = "ATTm590480";
const DS_SAMACCOUNT_TYPE_INDEX_NAME: &str = "ATTj590126";
const DS_USER_ACCOUNT_CONTROL_INDEX_NAME: &str = "ATTj589832";
const DS_LAST_LOGON_INDEX_NAME: &str = "ATTq589876";
const DS_LAST_LOGON_TIME_STAMP_INDEX_NAME: &str = "ATTq591520";
const DS_ACCOUNT_EXPIRES_INDEX_NAME: &str = "ATTq589983";
const DS_PASSWORD_LAST_SET_INDEX_NAME: &str = "ATTq589920";
const DS_BAD_PWD_TIME_INDEX_NAME: &str = "ATTq589873";
const DS_LOGON_COUNT_INDEX_NAME: &str = "ATTj589993";
const DS_BAD_PWD_COUNT_INDEX_NAME: &str = "ATTj589836";
const DS_PRIMARY_GROUP_ID_INDEX_NAME: &str = "ATTj589922";
const DS_NTHASH_INDEX_NAME: &str = "ATTk589914";
const DS_LMHASH_INDEX_NAME: &str = "ATTk589879";
const DS_NTHASH_HISTORY_INDEX_NAME: &str = "ATTk589918";
const DS_LMHASH_HISTORY_INDEX_NAME: &str = "ATTk589984";
const DS_UNIX_PASSWORD_INDEX_NAME: &str = "ATTk591734";
const DS_ADUSER_OBJECTS_INDEX_NAME: &str = "ATTk36";
const DS_SUPPLEMENTAL_CREDENTIALS_INDEX_NAME: &str = "ATTk589949";

const DS_DNSHOST_NAME_INDEX_NAME: &str = "ATTm590443";
const DS_OSNAME_INDEX_NAME: &str = "ATTm590187";
const DS_OSVERSION_INDEX_NAME: &str = "ATTm590188";

const DS_RECOVERY_PASSWORD_INDEX_NAME: &str = "ATTm591788";
const DS_FVEKEY_PACKAGE_INDEX_NAME: &str = "ATTk591823";
const DS_VOLUME_GUIDINDEX_NAME: &str = "ATTk591822";
const DS_RECOVERY_GUIDINDEX_NAME: &str = "ATTk591789";
const DS_DIAL_IN_ACCESS_PERMISSION_NAME: &str = "ATTi590943";
const DS_PEKINDEX_NAME: &str = "ATTk590689";

struct ColumnInfoMapping {
    pub (crate) dsRecordIdIndex: ColumnInformation,
    pub (crate) dsParentRecordIdIndex: ColumnInformation,
    pub (crate) dsRecordTimeIndex: ColumnInformation,
    pub (crate) dsAncestorsIndex: ColumnInformation,
    pub (crate) dsObjectTypeIdIndex: ColumnInformation,
    pub (crate) dsObjectNameIndex: ColumnInformation,
    pub (crate) dsObjectName2Index: ColumnInformation,
    pub (crate) dsObjectGUIDIndex: ColumnInformation,
    pub (crate) dsWhenCreatedIndex: ColumnInformation,
    pub (crate) dsWhenChangedIndex: ColumnInformation,
    pub (crate) dsUSNCreatedIndex: ColumnInformation,
    pub (crate) dsUSNChangedIndex: ColumnInformation,
    pub (crate) dsObjectColIndex: ColumnInformation,
    pub (crate) dsIsDeletedIndex: ColumnInformation,

    pub (crate) dsOrigContainerIdIndex: ColumnInformation,

    pub (crate) ds_sidindex: ColumnInformation,
    pub (crate) ds_samaccount_name_index: ColumnInformation,
    pub (crate) ds_user_principal_name_index: ColumnInformation,
    pub (crate) ds_samaccount_type_index: ColumnInformation,
    pub (crate) ds_user_account_control_index: ColumnInformation,
    pub (crate) ds_last_logon_index: ColumnInformation,
    pub (crate) ds_last_logon_time_stamp_index: ColumnInformation,
    pub (crate) ds_account_expires_index: ColumnInformation,
    pub (crate) ds_password_last_set_index: ColumnInformation,
    pub (crate) ds_bad_pwd_time_index: ColumnInformation,
    pub (crate) ds_logon_count_index: ColumnInformation,
    pub (crate) ds_bad_pwd_count_index: ColumnInformation,
    pub (crate) ds_primary_group_id_index: ColumnInformation,
    pub (crate) ds_nthash_index: ColumnInformation,
    pub (crate) ds_lmhash_index: ColumnInformation,
    pub (crate) ds_nthash_history_index: ColumnInformation,
    pub (crate) ds_lmhash_history_index: ColumnInformation,
    pub (crate) ds_unix_password_index: ColumnInformation,
    pub (crate) ds_aduser_objects_index: ColumnInformation,
    pub (crate) ds_supplemental_credentials_index: ColumnInformation,

    pub (crate) dsDNSHostNameIndex: ColumnInformation,
    pub (crate) dsOSNameIndex: ColumnInformation,
    pub (crate) dsOSVersionIndex: ColumnInformation,
    pub (crate) dsRecoveryPasswordIndex: ColumnInformation,
    pub (crate) dsFVEKeyPackageIndex: ColumnInformation,
    pub (crate) dsVolumeGUIDIndex: ColumnInformation,
    pub (crate) dsRecoveryGUIDIndex: ColumnInformation,
    pub (crate) dsDialInAccessPermission: ColumnInformation,
    pub (crate) dsPEKIndex: ColumnInformation,
}

fn main() -> Result<()> {
    let cli = Args::parse();
    let _ = TermLogger::init(
        cli.verbose.log_level_filter(), 
        Config::default(),
        simplelog::TerminalMode::Stderr,
        simplelog::ColorChoice::Auto);

    let ntds_path = Path::new(&cli.ntds_file);
    if ! (ntds_path.exists() && ntds_path.is_file()) {
        eprintln!("unable to open '{}'", cli.ntds_file);
        std::process::exit(-1);
    }

    let esedb = EseDb::open(&cli.ntds_file)?;
    log::info!("Db load finished");

    let data_table = esedb.table_by_name("datatable")?;

    let mut temporary_mapping = HashMap::new();
    for index in 0..data_table.count_columns()? {
        let column_res = data_table.column(index)?;
        let col_info = ColumnInformation::new(
            index,
            column_res.name()?,
            column_res.variant()?
        );
        temporary_mapping.insert(column_res.name()?, col_info);
    }

    let mapping = ColumnInfoMapping {
        dsRecordIdIndex: temporary_mapping.remove(DS_RECORD_ID_INDEX_NAME).unwrap(),
        dsParentRecordIdIndex: temporary_mapping.remove(DS_PARENT_RECORD_ID_INDEX_NAME).unwrap(),
        dsRecordTimeIndex: temporary_mapping.remove(DS_RECORD_TIME_INDEX_NAME).unwrap(),
        dsAncestorsIndex: temporary_mapping.remove(DS_ANCESTORS_INDEX_NAME).unwrap(),
        dsObjectTypeIdIndex: temporary_mapping.remove(DS_OBJECT_TYPE_ID_INDEX_NAME).unwrap(),
        dsObjectNameIndex: temporary_mapping.remove(DS_OBJECT_NAME_INDEX_NAME).unwrap(),
        dsObjectName2Index: temporary_mapping.remove(DS_OBJECT_NAME2_INDEX_NAME).unwrap(),
        dsObjectGUIDIndex: temporary_mapping.remove(DS_OBJECT_GUIDINDEX_NAME).unwrap(),
        dsWhenCreatedIndex: temporary_mapping.remove(DS_WHEN_CREATED_INDEX_NAME).unwrap(),
        dsWhenChangedIndex: temporary_mapping.remove(DS_WHEN_CHANGED_INDEX_NAME).unwrap(),
        dsUSNCreatedIndex: temporary_mapping.remove(DS_USNCREATED_INDEX_NAME).unwrap(),
        dsUSNChangedIndex: temporary_mapping.remove(DS_USNCHANGED_INDEX_NAME).unwrap(),
        dsObjectColIndex: temporary_mapping.remove(DS_OBJECT_COL_INDEX_NAME).unwrap(),
        dsIsDeletedIndex: temporary_mapping.remove(DS_IS_DELETED_INDEX_NAME).unwrap(),
        dsOrigContainerIdIndex: temporary_mapping.remove(DS_ORIG_CONTAINER_ID_INDEX_NAME).unwrap(),
        ds_sidindex: temporary_mapping.remove(DS_SIDINDEX_NAME).unwrap(),
        ds_samaccount_name_index: temporary_mapping.remove(DS_SAMACCOUNT_NAME_INDEX_NAME).unwrap(),
        ds_user_principal_name_index: temporary_mapping.remove(DS_USER_PRINCIPAL_NAME_INDEX_NAME).unwrap(),
        ds_samaccount_type_index: temporary_mapping.remove(DS_SAMACCOUNT_TYPE_INDEX_NAME).unwrap(),
        ds_user_account_control_index: temporary_mapping.remove(DS_USER_ACCOUNT_CONTROL_INDEX_NAME).unwrap(),
        ds_last_logon_index: temporary_mapping.remove(DS_LAST_LOGON_INDEX_NAME).unwrap(),
        ds_last_logon_time_stamp_index: temporary_mapping.remove(DS_LAST_LOGON_TIME_STAMP_INDEX_NAME).unwrap(),
        ds_account_expires_index: temporary_mapping.remove(DS_ACCOUNT_EXPIRES_INDEX_NAME).unwrap(),
        ds_password_last_set_index: temporary_mapping.remove(DS_PASSWORD_LAST_SET_INDEX_NAME).unwrap(),
        ds_bad_pwd_time_index: temporary_mapping.remove(DS_BAD_PWD_TIME_INDEX_NAME).unwrap(),
        ds_logon_count_index: temporary_mapping.remove(DS_LOGON_COUNT_INDEX_NAME).unwrap(),
        ds_bad_pwd_count_index: temporary_mapping.remove(DS_BAD_PWD_COUNT_INDEX_NAME).unwrap(),
        ds_primary_group_id_index: temporary_mapping.remove(DS_PRIMARY_GROUP_ID_INDEX_NAME).unwrap(),
        ds_nthash_index: temporary_mapping.remove(DS_NTHASH_INDEX_NAME).unwrap(),
        ds_lmhash_index: temporary_mapping.remove(DS_LMHASH_INDEX_NAME).unwrap(),
        ds_nthash_history_index: temporary_mapping.remove(DS_NTHASH_HISTORY_INDEX_NAME).unwrap(),
        ds_lmhash_history_index: temporary_mapping.remove(DS_LMHASH_HISTORY_INDEX_NAME).unwrap(),
        ds_unix_password_index: temporary_mapping.remove(DS_UNIX_PASSWORD_INDEX_NAME).unwrap(),
        ds_aduser_objects_index: temporary_mapping.remove(DS_ADUSER_OBJECTS_INDEX_NAME).unwrap(),
        ds_supplemental_credentials_index: temporary_mapping.remove(DS_SUPPLEMENTAL_CREDENTIALS_INDEX_NAME).unwrap(),
        dsDNSHostNameIndex: temporary_mapping.remove(DS_DNSHOST_NAME_INDEX_NAME).unwrap(),
        dsOSNameIndex: temporary_mapping.remove(DS_OSNAME_INDEX_NAME).unwrap(),
        dsOSVersionIndex: temporary_mapping.remove(DS_OSVERSION_INDEX_NAME).unwrap(),
        dsRecoveryPasswordIndex: temporary_mapping.remove(DS_RECOVERY_PASSWORD_INDEX_NAME).unwrap(),
        dsFVEKeyPackageIndex: temporary_mapping.remove(DS_FVEKEY_PACKAGE_INDEX_NAME).unwrap(),
        dsVolumeGUIDIndex: temporary_mapping.remove(DS_VOLUME_GUIDINDEX_NAME).unwrap(),
        dsRecoveryGUIDIndex: temporary_mapping.remove(DS_RECOVERY_GUIDINDEX_NAME).unwrap(),
        dsDialInAccessPermission: temporary_mapping.remove(DS_DIAL_IN_ACCESS_PERMISSION_NAME).unwrap(),
        dsPEKIndex: temporary_mapping.remove(DS_PEKINDEX_NAME).unwrap(),
    };

    log::info!("reading schema information and creating record cache");

    let mut schema_record = None;
    for record_res in data_table.iter_records()? {
        match record_res {
            Err(why) => {
                log::error!("unable to read record: {why}");
                break;
            }
            Ok(record) => {
                let dbrecord = DbRecord::from(record);
                let object_name2 = dbrecord.ds_object_name2_index(&mapping)?.expect("missing object_name2 attribute");

                // special handling for schema root object
                if object_name2 == "Schema" {
                    let schema_record_id = dbrecord.ds_record_id_index(&mapping)?.unwrap();
                    schema_record = Some(dbrecord);
                    log::debug!("found schema record id: {}", schema_record_id);
                    break;                    
                }
            }
        }
    }

    let schema_record = schema_record.expect("missing schema record");
    let schema_record_id = schema_record.ds_record_id_index(&mapping)?;

    let mut type_names = hashset!{
        "Person"
    };
    let mut type_records = HashMap::new();

    for dbrecord in data_table.iter_records()?
                                .filter_map(|r| r.ok()) 
                                .map(|r| DbRecord::from(r))
                                .filter(|dbrecord| dbrecord.ds_parent_record_id_index(&mapping).unwrap() == schema_record_id) {
                
        let object_name2 = dbrecord.ds_object_name2_index(&mapping)?.expect("missing object_name2 attribute");

        if type_names.remove(&object_name2[..]) {
            log::trace!("found type definition for '{object_name2}'");
            type_records.insert(object_name2, dbrecord);
        }

        if type_names.is_empty() {
            break;
        }
    }
    log::info!("found all necessary type definitions");

    let person_type_id = type_records.get("Person").unwrap().ds_record_id_index(&mapping)?;

    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    for person in data_table.iter_records()?
                                    .filter_map(|r| r.ok())
                                    .map(|r| DbRecord::from(r))
                                    .filter(|dbrecord| dbrecord.ds_object_type_id_index(&mapping).is_ok())
                                    .filter(|dbrecord| dbrecord.ds_object_type_id_index(&mapping).unwrap() == person_type_id)
                                    .map(|dbrecord| Person::from(dbrecord, &mapping).unwrap()){
        wtr.serialize(person)?;
    }
    drop(wtr);

    Ok(())
}
