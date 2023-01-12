use std::fmt;

use serde::{Serialize};
use serde::{Serializer};
use akaze::KeyPoint;
use serde::ser::SerializeStruct;
use serde::de::{self, Deserialize, Deserializer, Visitor, MapAccess, SeqAccess};

#[derive(Debug)]
pub struct CacheEntry {
    pub path: String,
    pub keypoints: Vec<MyKeyPoint>,
    pub descriptors: Vec<Vec<f32>>
}

struct CacheEntryVisitor;
struct MyKeyPointVisitor;

#[derive(Debug)]
pub struct MyKeyPoint(pub KeyPoint);

impl Serialize for CacheEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {

        let num_fields = 1 + self.keypoints.len()*6 + self.descriptors.len();

        let mut state = serializer.serialize_struct("CacheEntry", num_fields)?;

        /* serialize string */
        let _ = state.serialize_field("path", &self.path);

        /* serialize keypoints and descriptors */
        let mydescriptors: Vec<Vec<f32>> = Vec::from(self.descriptors.iter().map(|x| x.clone()).collect::<Vec<Vec<f32>>>());
        let _ = state.serialize_field("keypoints", &self.keypoints);
        let _ = state.serialize_field("descriptors", &mydescriptors);

        /* finalize  */
        state.end()
    }
}

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

impl<'de> Deserialize<'de> for CacheEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field { Path, Keypoints, Descriptors }

        // This part could also be generated independently by:
        //
        //    #[derive(Deserialize)]
        //    #[serde(field_identifier, rename_all = "lowercase")]
        //    enum Field { Secs, Nanos }
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("'path' or 'keypoints' or 'descriptors'")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "path" => Ok(Field::Path),
                            "keypoints" => Ok(Field::Keypoints),
                            "descriptors" => Ok(Field::Descriptors),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        impl<'de> Visitor<'de> for CacheEntryVisitor {
            type Value = CacheEntry;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a CacheEntry struct")
            }

            fn visit_map<V>(self, mut map: V) -> Result<CacheEntry, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut path: Option<String> = None;
                let mut keypoints: Option<Vec<MyKeyPoint>> = None;
                let mut descriptors: Option<Vec<Vec<f32>>> = None;
                
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Path => {
                            if path.is_some() {
                                return Err(de::Error::duplicate_field("path"));
                            }
                            path = Some(map.next_value()?);
                            // println!("path: {}", path.clone().unwrap());
                        }
                        Field::Keypoints => {
                            if keypoints.is_some() {
                                return Err(de::Error::duplicate_field("keypoints"));
                            }
                            keypoints = Some(map.next_value()?);
                        },
                        Field::Descriptors => {
                            if descriptors.is_some() {
                                return Err(de::Error::duplicate_field("descriptors"));
                            }
                            descriptors = Some(map.next_value()?);
                        }
                    }
                }
                let path: String = path.ok_or_else(|| de::Error::missing_field("path"))?;
                let keypoints: Vec<MyKeyPoint> = keypoints.ok_or_else(|| de::Error::missing_field("keypoints"))?;
                let descriptors: Vec<Vec<f32>> = descriptors.ok_or_else(|| de::Error::missing_field("descriptors"))?;

                /* "unwrap" KeyPoints from MyKeyPoint wrappers */
                let descriptors: Vec<Vec<f32>> = descriptors.iter().map(|x| x.clone()).collect();

                Ok(CacheEntry { path, keypoints, descriptors })
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<CacheEntry, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let path = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let keypoints = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let descriptors = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                Ok(CacheEntry { path, keypoints, descriptors })
            }

        }

        const FIELDS: &'static [&'static str] = &["path", "keypoints", "descriptors"];
        deserializer.deserialize_struct("Duration", FIELDS, CacheEntryVisitor)
    }
}




impl<'de> Deserialize<'de> for MyKeyPoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
    
        enum Field { Point, Response, Size, Octave, ClassID, Angle }
        // This part could also be generated independently by:
        //
        //    #[derive(Deserialize)]
        //    #[serde(field_identifier, rename_all = "lowercase")]
        //    enum Field { Secs, Nanos }
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("a MyKeyPoint")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            // Point, Response, Size, Octave, ClassID, Angle
                            "point" => Ok(Field::Point),
                            "response" => Ok(Field::Response),
                            "size" => Ok(Field::Size),
                            "octave" => Ok(Field::Octave),
                            "class_id" => Ok(Field::ClassID),
                            "angle" => Ok(Field::Angle),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        impl<'de> Visitor<'de> for MyKeyPointVisitor {
            type Value = MyKeyPoint;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct MyKeyPoint")
            }

            fn visit_map<V>(self, mut map: V) -> Result<MyKeyPoint, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut point: Option<(f32, f32)> = None;
                let mut response: Option<f32> = None;
                let mut size: Option<f32> = None;
                let mut octave: Option<usize> = None;
                let mut class_id: Option<usize> = None;
                let mut angle: Option<f32> = None;
                
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Point => {
                            if point.is_some() {
                                return Err(de::Error::duplicate_field("path"));
                            }
                            point = Some(map.next_value()?);
                        }
                        Field::Response => {
                            if response.is_some() {
                                return Err(de::Error::duplicate_field("path"));
                            }
                            response = Some(map.next_value()?);
                        }
                        Field::Size => {
                            if size.is_some() {
                                return Err(de::Error::duplicate_field("path"));
                            }
                            size = Some(map.next_value()?);
                        }
                        Field::Octave => {
                            if octave.is_some() {
                                return Err(de::Error::duplicate_field("path"));
                            }
                            octave = Some(map.next_value()?);
                        }
                        Field::ClassID => {
                            if class_id.is_some() {
                                return Err(de::Error::duplicate_field("path"));
                            }
                            class_id = Some(map.next_value()?);
                        }
                        Field::Angle => {
                            if angle.is_some() {
                                return Err(de::Error::duplicate_field("path"));
                            }
                            angle = Some(map.next_value()?);
                        }
                    }
                }

                let point = point.ok_or_else(|| de::Error::missing_field("point"))?;
                let response = response.ok_or_else(|| de::Error::missing_field("response"))?;
                let size = size.ok_or_else(|| de::Error::missing_field("size"))?;
                let octave = octave.ok_or_else(|| de::Error::missing_field("octave"))?;
                let class_id = class_id.ok_or_else(|| de::Error::missing_field("class_id"))?;
                let angle = angle.ok_or_else(|| de::Error::missing_field("angle"))?;

                Ok(MyKeyPoint(KeyPoint { point, response, size, octave, class_id, angle }))
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<MyKeyPoint, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let point = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let response = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let size = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let octave = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                let class_id = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;
                let angle = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?;
                Ok(MyKeyPoint(KeyPoint { point, response, size, octave, class_id, angle }))
            }
        }

        const FIELDS: &'static [&'static str] = &["point", "response", "size", "octave", "class_id", "angle"];
        deserializer.deserialize_struct("MyKeyPoint", FIELDS, MyKeyPointVisitor)
    }
}


