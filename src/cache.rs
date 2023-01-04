use crate::feature_matching::bitarray_to_floatarray;

use serde::{Serialize, Deserialize};
use serde::{Serializer, Deserializer};
use akaze::KeyPoint;
use bitarray::BitArray;
use serde::ser::SerializeStruct;

#[derive(Debug)]
pub struct CacheEntry {
    path: String,
    keypoints: Vec<KeyPoint>,
    descriptors: Vec<BitArray<64>>
}

pub struct MyKeyPoint(KeyPoint);

impl Serialize for MyKeyPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut state = serializer.serialize_struct("CacheEntry", 6)?;

        let _ = state.serialize_field("point", &self.0.point);
        let _ = state.serialize_field("response", &self.0.response);
        let _ = state.serialize_field("size", &self.0.size);
        let _ = state.serialize_field("octave", &self.0.octave);
        let _ = state.serialize_field("class_id", &self.0.class_id);
        let _ = state.serialize_field("angle", &self.0.angle);

        state.end()
    }
}

impl Serialize for CacheEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let num_keypoints = self.keypoints.len();
        let num_descriptors = self.descriptors.len();
        let len = 1 + num_keypoints*6 + num_descriptors;

        let mut state = serializer.serialize_struct("CacheEntry", len)?;

        /* serialize string */
        let _ = state.serialize_field("path", &self.path);

        /* serialize keypoints and descriptors */
        let mykeypoints: Vec<MyKeyPoint> = self.keypoints.iter().map(|x| MyKeyPoint(*x)).collect();
        let mydescriptors: Vec<Vec<f32>> = self.descriptors.iter().map(|x| bitarray_to_floatarray(x)).collect();
        let _ = state.serialize_field("keypoints", &mykeypoints);
        let _ = state.serialize_field("descriptors", &mydescriptors);

        /* finalize  */
        state.end()
    }
}
