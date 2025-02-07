use std::collections::HashMap;

use base64::prelude::*;

use crate::{
    cache::{self, Value, WithValue},
    win32_types::SecurityDescriptor,
};

pub struct SdTable {
    descriptors: HashMap<i64, Vec<u8>>,
}

impl SdTable {
    pub fn new(sd_table: &cache::SdTable) -> crate::ntds::Result<Self> {
        let sd_id_column = sd_table.sd_id_column();
        let sd_value_column = sd_table.sd_value_column();

        let descriptors = sd_table
            .iter()
            .map(|record| {
                let sd_id = record
                    .with_value(*sd_id_column, |v| match v.unwrap() {
                        Value::I16(v) => Ok(i64::from(*v)),
                        Value::I32(v) => Ok(i64::from(*v)),
                        Value::I64(v) => Ok(*v),
                        Value::Currency(v) => Ok(*v),
                        v => unimplemented!("no support for {v} as sd_id"),
                    })
                    .unwrap();
                let sd_value = record
                    .with_value(*sd_value_column, |v| match v.unwrap() {
                        Value::Long(v) | Value::Binary(v) | Value::LargeBinary(v) => {
                            Ok(v.as_ref().clone())
                        }
                        v => unimplemented!("no support for {v} as sd_value"),
                    })
                    .unwrap();

                (sd_id, sd_value)
            })
            .collect();
        Ok(Self { descriptors })
    }

    pub fn descriptor(
        &self,
        sd_id: &i64,
    ) -> Option<Result<SecurityDescriptor, crate::ntds::Error>> {
        self.descriptors
            .get(sd_id)
            .map(|v| match SecurityDescriptor::try_from(&v[..]) {
                Ok(sd) => Ok(sd),
                Err(why) => {
                    log::error!("failed descriptor was: {}", BASE64_STANDARD.encode(v));
                    log::error!("{why}");
                    Err(why)
                }
            })
    }
}
