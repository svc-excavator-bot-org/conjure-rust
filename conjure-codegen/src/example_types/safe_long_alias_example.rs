use conjure_types::serde::{de, ser};
#[derive(Debug, Clone, PartialEq, PartialOrd, Copy, Eq, Ord, Hash, Default)]
pub struct SafeLongAliasExample(pub conjure_types::SafeLong);
impl std::ops::Deref for SafeLongAliasExample {
    type Target = conjure_types::SafeLong;
    #[inline]
    fn deref(&self) -> &conjure_types::SafeLong {
        &self.0
    }
}
impl std::ops::DerefMut for SafeLongAliasExample {
    #[inline]
    fn deref_mut(&mut self) -> &mut conjure_types::SafeLong {
        &mut self.0
    }
}
impl ser::Serialize for SafeLongAliasExample {
    fn serialize<S_>(&self, s: S_) -> Result<S_::Ok, S_::Error>
    where
        S_: ser::Serializer,
    {
        self.0.serialize(s)
    }
}
impl<'de> de::Deserialize<'de> for SafeLongAliasExample {
    fn deserialize<D_>(d: D_) -> Result<SafeLongAliasExample, D_::Error>
    where
        D_: de::Deserializer<'de>,
    {
        de::Deserialize::deserialize(d).map(SafeLongAliasExample)
    }
}