use super::sample_info::SampleInfo;

#[derive(Debug)]
pub struct DataSample<T> {
    pub infos: SampleInfo,
    pub data: Option<T>,
}

impl<T> DataSample<T> {
    pub(crate) fn new(infos: SampleInfo, data: Option<T>) -> Self {
        Self { infos, data }
    }

    pub fn data(&self) -> Option<&T> {
        self.data.as_ref()
    }

    pub fn get_data_mut(&mut self) -> Option<&mut T> {
        self.data.as_mut()
    }

    pub fn take_data(self) -> Option<T> {
        self.data
    }

    pub fn infos(&self) -> SampleInfo {
        self.infos
    }
}

impl<T> Default for DataSample<T> {
    fn default() -> Self {
        Self {
            infos: Default::default(),
            data: Default::default(),
        }
    }
}
