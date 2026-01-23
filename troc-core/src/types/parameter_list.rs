use std::{
    fmt::{Debug, Display},
    io::{BufWriter, Cursor, Seek, Write},
    mem::{size_of, size_of_val},
};

use binrw::{BinRead, BinResult, BinWrite, Endian, binrw};
use cdr::{Error, Infinite};
use serde::{Deserialize, Serialize};

use crate::types::{
    InstanceHandle, SerializedData, parameter::PID_PAD_ID, parameter_id::ParameterId,
};

use super::{
    DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, Guid, HistoryQosPolicy,
    LifespanQosPolicy, LivelinessQosPolicy, ReliabilityQosPolicy, ResourceLimitsQosPolicy,
    parameter::{PID_SENTINEL_ID, Parameter},
    status_info::StatusInfo,
    time_based_filter_qos::TimeBasedFilterQosPolicy,
    timestamp::Timestamp,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[binrw]
pub struct ParameterList {
    #[br(parse_with = parameter_list_parser)]
    #[bw(write_with = parameter_list_writer)]
    parameters: Vec<Parameter>,
}

impl ParameterList {
    pub fn get_param_raw(&self, parameter_id: ParameterId) -> Option<Vec<u8>> {
        self.parameters
            .iter()
            .find(|p| p.parameter_id == parameter_id)
            .map(|p| p.value.clone())
    }

    pub fn get_param<T>(&self, parameter_id: ParameterId, endian: Endian) -> Option<T>
    where
        T: BinRead,
        for<'a> T: BinRead<Args<'a> = (usize,)>,
    {
        if let Some(p) = self
            .parameters
            .iter()
            .find(|p| p.parameter_id == parameter_id)
        {
            let mut reader = Cursor::new(&p.value);
            let res = T::read_options(&mut reader, endian, (p.length as usize,));
            Some(res.unwrap())
        } else {
            None
        }
    }

    pub fn get_params<T>(&self, parameter_id: ParameterId, endian: Endian) -> Vec<T>
    where
        T: BinRead,
        for<'a> T: BinRead<Args<'a> = ()>,
    {
        self.parameters
            .iter()
            .filter(|p| p.parameter_id == parameter_id)
            .map(|p| {
                let mut reader = Cursor::new(&p.value);
                T::read_options(&mut reader, endian, ()).unwrap()
            })
            .collect()
    }

    pub fn set_param<T>(&mut self, param_id: ParameterId, param: T, endian: Endian)
    where
        for<'a> T: BinWrite<Args<'a> = ()>,
    {
        let mut buffer = Vec::with_capacity(u16::MAX as usize);
        let mut writer = Cursor::new(&mut buffer);
        param.write_options(&mut writer, endian, ()).unwrap();
        let param = Parameter::new(param_id, &buffer);
        if param.value.is_empty() {
            return;
        };
        self.parameters.push(param);
    }

    pub fn merge(&mut self, other: &ParameterList) {
        self.parameters.extend_from_slice(&other.parameters);
    }

    pub fn terminate(&mut self) -> Self {
        self.to_owned()
    }

    pub fn new() -> Self {
        let parameters = Vec::default();
        // let buffer = Vec::with_capacity(u16::MAX as usize);
        Self { parameters }
    }

    pub fn size(&self) -> usize {
        let buf = Vec::new();
        let mut writer = Cursor::new(buf);
        ParameterList::write_be(self, &mut writer).unwrap();
        writer.into_inner().len()
    }

    pub fn remove(&mut self, param: Parameter) {
        self.parameters.retain(|e| e.ne(&param))
    }

    pub fn get(&self, id: ParameterId) -> Option<&Parameter> {
        self.parameters
            .iter()
            .find(|param| param.parameter_id == id)
    }

    pub fn get_mut(&mut self, id: ParameterId) -> Option<&mut Parameter> {
        self.parameters
            .iter_mut()
            .find(|param| param.parameter_id == id)
    }

    pub fn add_or_update(&mut self, param: Parameter) {
        if let Some(in_param) = self
            .parameters
            .iter_mut()
            .find(|p| p.parameter_id == param.parameter_id)
        {
            *in_param = param;
        } else {
            self.parameters.push(param);
        }
    }

    pub fn from_serialized_data(data: SerializedData) -> Result<(ParameterList, Endian), Error> {
        let endian = match data.get_data()[1] {
            0x03 => Endian::Little,
            0x02 => Endian::Big,
            _ => panic!(),
        };
        let mut reader = Cursor::new(&data.get_data()[4..]);
        let param_list = ParameterList::read_options(&mut reader, endian, ()).unwrap();
        Ok((param_list, endian))
    }

    pub fn into_serialized_data(&self, endian: Endian) -> Result<SerializedData, Error> {
        let buf = Vec::<u8>::new();
        let mut writer = Cursor::new(buf);
        ParameterList::write_options(self, &mut writer, endian, ()).unwrap();
        let buf = writer.into_inner();

        let cdr_header: &[u8] = match endian {
            Endian::Big => &[0x00, 0x02, 0x00, 0x00],
            Endian::Little => &[0x00, 0x03, 0x00, 0x00],
        };

        let buf = [cdr_header, &buf].concat();

        Ok(SerializedData::from_slice(&buf))
    }
}

#[binrw::parser(reader, endian)]
fn parameter_list_parser() -> BinResult<Vec<Parameter>> {
    let mut parameters = Vec::new();
    let mut buffer_id_length: [u8; 2] = [0; 2];

    loop {
        reader.read_exact(&mut buffer_id_length)?;
        let parameter_id = match endian {
            Endian::Little => ParameterId(i16::from_le_bytes(buffer_id_length)),
            Endian::Big => ParameterId(i16::from_be_bytes(buffer_id_length)),
        };

        reader.read_exact(&mut buffer_id_length)?;
        let length = match endian {
            Endian::Little => i16::from_le_bytes(buffer_id_length),
            Endian::Big => i16::from_be_bytes(buffer_id_length),
        };

        if parameter_id == PID_SENTINEL_ID {
            break;
        }

        let mut value = vec![0u8; length as usize];
        reader.read_exact(&mut value)?;

        if parameter_id == PID_PAD_ID {
            continue;
        }

        let parameter = Parameter {
            parameter_id,
            length,
            value,
        };

        parameters.push(parameter);
    }

    Ok(parameters)
}

#[allow(clippy::ptr_arg)]
#[binrw::writer(writer, endian)]
fn parameter_list_writer(parameters: &Vec<Parameter>) -> BinResult<()> {
    for mut param in parameters.clone() {
        let pad_length = (4 - (param.length % 4)) % 4;
        param.length += pad_length;

        param.write_options(writer, endian, ())?;

        for _ in 0..pad_length {
            let _ = writer.write(&[0u8]);
        }
    }

    let sentinel = Parameter {
        parameter_id: PID_SENTINEL_ID,
        length: 0,
        value: vec![],
    };
    sentinel.write_options(writer, endian, ())?;

    Ok(())
}

impl Display for ParameterList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{")?;
        f.write_str(&format!("{:?}, ", self.parameters))?;
        f.write_str("}")?;
        Ok(())
    }
}
