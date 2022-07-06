use libesedb::{self, Value};
use anyhow::{anyhow, Result};
use std::io::Cursor;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use chrono::{DateTime, Utc, TimeZone, Duration, NaiveDate};

use crate::ColumnInfoMapping;

macro_rules! define_i32_getter {
    ($fn_name: ident, $mapping_name: ident) => {
        
    pub fn $fn_name(&self, mapping: &ColumnInfoMapping) -> Result<Option<i32>> {
        let value = self.inner_record.value(mapping.$mapping_name.id)?;
        match value {
            Value::I32(val) => Ok(Some(val)),
            Value::Null => Ok(None),
            _ => Err(anyhow!("invalid value detected: {:?} in field {}", value, stringify!($fn_name)))
        }
    }
    };
}

macro_rules! define_str_getter {
    ($fn_name: ident, $mapping_name: ident) => {
        
    pub fn $fn_name(&self, mapping: &ColumnInfoMapping) -> Result<Option<String>> {
        let value = self.inner_record.value(mapping.$mapping_name.id)?;
        match value {
            Value::Text(val) => Ok(Some(val)),
            Value::LargeText(val) => Ok(Some(val)),
            Value::Null => Ok(None),
            _ => Err(anyhow!("invalid value detected: {:?} in field {}", value, stringify!($fn_name)))
        }
    }
    };
}

macro_rules! define_bin_getter {
    ($fn_name: ident, $mapping_name: ident) => {
        
    pub fn $fn_name(&self, mapping: &ColumnInfoMapping) -> Result<Option<String>> {
        let value = self.inner_record.value(mapping.$mapping_name.id)?;
        match value {
            Value::Binary(val) | Value::LargeBinary(val) => {
                Ok(Some(hex::encode(val)))
            }
            Value::Null => Ok(None),
            _ => Err(anyhow!("invalid value detected: {:?} in field {}", value, stringify!($fn_name)))
        }
    }
    };
}


/// https://devblogs.microsoft.com/oldnewthing/20040315-00/?p=40253
macro_rules! define_sid_getter {
    ($fn_name: ident, $mapping_name: ident) => {
        
    pub fn $fn_name(&self, mapping: &ColumnInfoMapping) -> Result<Option<String>> {
        let value = self.inner_record.value(mapping.$mapping_name.id)?;
        match value {
            Value::Binary(val) | Value::LargeBinary(val) => {
                //log::debug!("val: {:?}", val);
                let mut rdr = Cursor::new(val);
                let revision = rdr.read_u8()?;
                let number_of_dashes = rdr.read_u8()?;
                let authority = rdr.read_u48::<BigEndian>()?;

                //log::debug!("authority: {:012x}", authority);

                let mut numbers = vec![];
                for _i in 0..number_of_dashes-1 {
                    numbers.push(rdr.read_u32::<LittleEndian>()?);
                }
                numbers.push(rdr.read_u32::<BigEndian>()?);

                let numbers = numbers
                    .into_iter()
                    .map(|n| format!("{n}")).collect::<Vec<String>>().join("-");

                Ok(Some(format!("S-{revision}-{authority}-{numbers}")))
            }
            Value::Null => Ok(None),
            _ => Err(anyhow!("invalid value detected: {:?} in field {}", value, stringify!($fn_name)))
        }
    }
    };
}

macro_rules! define_datetime_getter {
    ($fn_name: ident, $mapping_name: ident) => {
        
    pub fn $fn_name(&self, mapping: &ColumnInfoMapping) -> Result<Option<DateTime<Utc>>> {
        let dt_base = DateTime::<Utc>::from_utc(NaiveDate::from_ymd(1601, 1, 1).and_hms(0, 0, 0), Utc);
        let value = self.inner_record.value(mapping.$mapping_name.id)?;
        match value {
            Value::Currency(val) => {

                let duration = Duration::microseconds(val / 10);
                Ok(Some(dt_base + duration))
            }
            Value::Null => Ok(None),
            _ => Err(anyhow!("invalid value detected: {:?} in field {}", value, stringify!($fn_name)))
        }
    }
    };
}

pub (crate) struct DbRecord<'a> {
    inner_record: libesedb::Record<'a>,
}

impl<'a> From<libesedb::Record<'a>> for DbRecord<'a> {
    fn from(inner: libesedb::Record<'a>) -> Self {
        Self {
            inner_record: inner
        }
    }
}

impl<'a> DbRecord<'a> {
    define_i32_getter!(ds_record_id_index, dsRecordIdIndex);
    define_i32_getter!(ds_parent_record_id_index, dsParentRecordIdIndex);

    pub fn ds_record_time_index(&self, mapping: &ColumnInfoMapping) -> Result<libesedb::Value, std::io::Error> {
        self.inner_record.value(mapping.dsRecordTimeIndex.id)
    }
    pub fn ds_ancestors_index(&self, mapping: &ColumnInfoMapping) -> Result<libesedb::Value, std::io::Error> {
        self.inner_record.value(mapping.dsAncestorsIndex.id)
    }
    define_i32_getter!(ds_object_type_id_index, dsObjectTypeIdIndex);

    define_str_getter!(ds_object_name_index, dsObjectNameIndex);
    define_str_getter!(ds_object_name2_index, dsObjectName2Index);

    define_sid_getter!(ds_sidindex, ds_sidindex);
    define_str_getter!(ds_samaccount_name_index, ds_samaccount_name_index);
    define_str_getter!(ds_user_principal_name_index, ds_user_principal_name_index);
    define_i32_getter!(ds_samaccount_type_index, ds_samaccount_type_index);
    define_i32_getter!(ds_user_account_control_index, ds_user_account_control_index);
    define_datetime_getter!(ds_last_logon_index, ds_last_logon_index);
    define_datetime_getter!(ds_last_logon_time_stamp_index, ds_last_logon_time_stamp_index);
    define_datetime_getter!(ds_account_expires_index, ds_account_expires_index);
    define_datetime_getter!(ds_password_last_set_index, ds_password_last_set_index);
    define_datetime_getter!(ds_bad_pwd_time_index, ds_bad_pwd_time_index);
    define_i32_getter!(ds_logon_count_index, ds_logon_count_index);
    define_i32_getter!(ds_bad_pwd_count_index, ds_bad_pwd_count_index);
    define_i32_getter!(ds_primary_group_id_index, ds_primary_group_id_index);
    define_bin_getter!(ds_nthash_index, ds_nthash_index);
    define_bin_getter!(ds_lmhash_index, ds_lmhash_index);
    define_bin_getter!(ds_nthash_history_index, ds_nthash_history_index);
    define_bin_getter!(ds_lmhash_history_index, ds_lmhash_history_index);
    define_str_getter!(ds_unix_password_index, ds_unix_password_index);
    define_bin_getter!(ds_aduser_objects_index, ds_aduser_objects_index);
    define_bin_getter!(ds_supplemental_credentials_index, ds_supplemental_credentials_index);
}