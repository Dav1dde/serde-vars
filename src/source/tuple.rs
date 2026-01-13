use crate::source::{Expansion, Source};

macro_rules! impl_tuple {
    ($($T:ident),*) => {
        #[allow(nonstandard_style)]
        impl<$($T,)*> Source for ($($T,)*)
        where
            $($T: Source,)*
        {
            fn expand_bool<E>(&mut self, v: &str) -> Result<Option<bool>, E>
            where
                E: serde::de::Error,
            {
                let ($($T,)*) = self;
                $(if let Some(value) = $T.expand_bool(v)? { return Ok(Some(value)); })*
                Ok(None)
            }

            fn expand_i8<E>(&mut self, v: &str) -> Result<Option<i8>, E>
            where
                E: serde::de::Error,
            {
                let ($($T,)*) = self;
                $(if let Some(value) = $T.expand_i8(v)? { return Ok(Some(value)); })*
                Ok(None)
            }

            fn expand_i16<E>(&mut self, v: &str) -> Result<Option<i16>, E>
            where
                E: serde::de::Error,
            {
                let ($($T,)*) = self;
                $(if let Some(value) = $T.expand_i16(v)? { return Ok(Some(value)); })*
                Ok(None)
            }

            fn expand_i32<E>(&mut self, v: &str) -> Result<Option<i32>, E>
            where
                E: serde::de::Error,
            {
                let ($($T,)*) = self;
                $(if let Some(value) = $T.expand_i32(v)? { return Ok(Some(value)); })*
                Ok(None)
            }

            fn expand_i64<E>(&mut self, v: &str) -> Result<Option<i64>, E>
            where
                E: serde::de::Error,
            {
                let ($($T,)*) = self;
                $(if let Some(value) = $T.expand_i64(v)? { return Ok(Some(value)); })*
                Ok(None)
            }

            fn expand_u8<E>(&mut self, v: &str) -> Result<Option<u8>, E>
            where
                E: serde::de::Error,
            {
                let ($($T,)*) = self;
                $(if let Some(value) = $T.expand_u8(v)? { return Ok(Some(value)); })*
                Ok(None)
            }

            fn expand_u16<E>(&mut self, v: &str) -> Result<Option<u16>, E>
            where
                E: serde::de::Error,
            {
                let ($($T,)*) = self;
                $(if let Some(value) = $T.expand_u16(v)? { return Ok(Some(value)); })*
                Ok(None)
            }

            fn expand_u32<E>(&mut self, v: &str) -> Result<Option<u32>, E>
            where
                E: serde::de::Error,
            {
                let ($($T,)*) = self;
                $(if let Some(value) = $T.expand_u32(v)? { return Ok(Some(value)); })*
                Ok(None)
            }

            fn expand_u64<E>(&mut self, v: &str) -> Result<Option<u64>, E>
            where
                E: serde::de::Error,
            {
                let ($($T,)*) = self;
                $(if let Some(value) = $T.expand_u64(v)? { return Ok(Some(value)); })*
                Ok(None)
            }

            fn expand_f32<E>(&mut self, v: &str) -> Result<Option<f32>, E>
            where
                E: serde::de::Error,
            {
                let ($($T,)*) = self;
                $(if let Some(value) = $T.expand_f32(v)? { return Ok(Some(value)); })*
                Ok(None)
            }

            fn expand_f64<E>(&mut self, v: &str) -> Result<Option<f64>, E>
            where
                E: serde::de::Error,
            {
                let ($($T,)*) = self;
                $(if let Some(value) = $T.expand_f64(v)? { return Ok(Some(value)); })*
                Ok(None)
            }

            fn expand_str<'a, E>(
                &mut self,
                v: std::borrow::Cow<'a, str>,
            ) -> Result<super::Expansion<std::borrow::Cow<'a, str>>, E>
            where
                E: serde::de::Error,
            {
                let ($($T,)*) = self;
                $(let v = match $T.expand_str(v)? {
                    Expansion::Expanded(v) => return Ok(Expansion::Expanded(v)),
                    Expansion::Original(v) => v,
                };)*
                Ok(Expansion::Original(v))
            }

            fn expand_bytes<'a, E>(
                &mut self,
                v: std::borrow::Cow<'a, [u8]>,
            ) -> Result<super::Expansion<std::borrow::Cow<'a, [u8]>>, E>
            where
                E: serde::de::Error,
            {
                let ($($T,)*) = self;
                $(let v = match $T.expand_bytes(v)? {
                    Expansion::Expanded(v) => return Ok(Expansion::Expanded(v)),
                    Expansion::Original(v) => v,
                };)*
                Ok(Expansion::Original(v))
            }

            fn expand_any<'a, E>(
                &mut self,
                v: std::borrow::Cow<'a, str>,
            ) -> Result<super::Expansion<super::Any<'a>, std::borrow::Cow<'a, str>>, E>
            where
                E: serde::de::Error,
            {
                let ($($T,)*) = self;
                $(let v = match $T.expand_any(v)? {
                    Expansion::Expanded(v) => return Ok(Expansion::Expanded(v)),
                    Expansion::Original(v) => v,
                };)*
                Ok(Expansion::Original(v))
            }
        }
    };
}

impl_tuple!(T1);
impl_tuple!(T1, T2);
impl_tuple!(T1, T2, T3);
impl_tuple!(T1, T2, T3, T4);
impl_tuple!(T1, T2, T3, T4, T5);
impl_tuple!(T1, T2, T3, T4, T5, T6);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
