use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListType {
    item_type: Box<super::Type>,
}
impl ListType {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new(item_type: super::Type) -> ListType {
        ListType {
            item_type: Box::new(item_type),
        }
    }
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> BuilderStage0 {
        Default::default()
    }
    #[inline]
    pub fn item_type(&self) -> &super::Type {
        &*self.item_type
    }
}
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {}
    }
}
impl From<ListType> for BuilderStage1 {
    #[inline]
    fn from(value: ListType) -> Self {
        BuilderStage1 {
            item_type: value.item_type,
        }
    }
}
///The stage 0 builder for the [`ListType`] type
#[derive(Debug, Clone)]
pub struct BuilderStage0 {}
impl BuilderStage0 {
    #[inline]
    pub fn item_type(self, item_type: super::Type) -> BuilderStage1 {
        BuilderStage1 {
            item_type: Box::new(item_type),
        }
    }
}
///The stage 1 builder for the [`ListType`] type
#[derive(Debug, Clone)]
pub struct BuilderStage1 {
    item_type: Box<super::Type>,
}
impl BuilderStage1 {
    #[inline]
    pub fn item_type(mut self, item_type: super::Type) -> Self {
        self.item_type = Box::new(item_type);
        self
    }
    /// Consumes the builder, constructing a new instance of the type.
    #[inline]
    pub fn build(self) -> ListType {
        ListType {
            item_type: self.item_type,
        }
    }
}
impl ser::Serialize for ListType {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let size = 1usize;
        let mut s = s.serialize_struct("ListType", size)?;
        s.serialize_field("itemType", &self.item_type)?;
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for ListType {
    fn deserialize<D>(d: D) -> Result<ListType, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct("ListType", &["itemType"], Visitor_)
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = ListType;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<ListType, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut item_type = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::ItemType => item_type = Some(map_.next_value()?),
                Field_::Unknown_ => {
                    map_.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        let item_type = match item_type {
            Some(v) => v,
            None => return Err(de::Error::missing_field("itemType")),
        };
        Ok(ListType { item_type })
    }
}
enum Field_ {
    ItemType,
    Unknown_,
}
impl<'de> de::Deserialize<'de> for Field_ {
    fn deserialize<D>(d: D) -> Result<Field_, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_str(FieldVisitor_)
    }
}
struct FieldVisitor_;
impl<'de> de::Visitor<'de> for FieldVisitor_ {
    type Value = Field_;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("string")
    }
    fn visit_str<E>(self, value: &str) -> Result<Field_, E>
    where
        E: de::Error,
    {
        let v = match value {
            "itemType" => Field_::ItemType,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
